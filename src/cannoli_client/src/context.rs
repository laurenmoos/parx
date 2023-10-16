use std::io::Write;
use crate::sinks::{SinkAction};
pub use crate::utils::utils::SymOff;
use serde::{Deserialize, Serialize};
use serde::ser::{Serializer, SerializeStruct};
use std::sync::Mutex;
use std::os::unix::net::UnixStream;
use multimap::MultiMap;
use crate::tracer::*;

pub use crate::utils::backtrace::{
    PidBacktraceCtx,
    BacktraceState,
};

pub struct TidContext {                                                             
    // Lookup from an address to a symbol, stored in sorted order               
    pub symbols: Vec<(u64, &'static str)>,
    pub tracer_ctx: TidTracerCtx,
    pub bitness: u8,
}

impl TidContext {
    // Attempt to resolve a symbol into a symbol and an offset
    pub fn resolve(&self, addr: u64) -> SymOff {
        // Get access to the symbols
        let symbols = &self.symbols;

        // Find the symbol
        match symbols.binary_search_by_key(&addr, |x| x.0) {
            Ok(pos) => {
                // Direct symbol match
                SymOff {
                    addr,
                    symbol: symbols[pos].1,
                    offset: 0
                }
            }
            Err(pos) => {
                // Found location after symbol, find the nearest symbol below
                if let Some((sa, sym)) = pos.checked_sub(1)
                        .and_then(|x| symbols.get(x)) {
                    // Got symbol below
                    SymOff {
                        addr,
                        symbol: sym,
                        offset: addr - sa
                    }
                } else {
                    // No symbols below this address, just emit the PC
                    SymOff {
                        addr,
                        symbol: "<unknown>",
                        offset: addr,
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Fingerprint {
    // sink: SinkDefinition,
    pub func_addr: u64,
    pub stack_offset: u64,
    pub data_type: DataType,
}

#[derive(Debug, Serialize)]
pub enum DataType {
    Ptr { addr: u64, val: u64, data: Vec<u8> },
    Int { addr: u64, val: u64},
    Bool { addr: u64, val: bool },
}

impl DataType {
    pub fn update(&mut self, access: &SimpleAccess) {
        println!("updating {:?}", self);
        match self {
            DataType::Ptr { addr: _, val, data } => {
                *val = access.value
            },
            DataType::Int { addr: _, val } => *val = access.value,
            _ => panic!("unhandled data type in DataType::update()")
        }
    }
}

#[derive(Debug)]
pub struct VariableCtx {
    pub fingerprints: Vec<Fingerprint>,
    pub vars: MultiMap<u64, DataType>
}

#[derive(Debug)]
pub struct PidVariableCtx(pub Mutex<VariableCtx>);

impl PidVariableCtx {
    pub fn is_fingerprinted(&self, pc: u64) -> bool {
        let variable_state: &mut VariableCtx = &mut self.0.lock().unwrap();
        for f in &variable_state.fingerprints {
            if pc == f.func_addr { return true }  
        }
        return false
    }

    pub fn fingerprint(&self, pc: u64, sp: u64) {
        let variable_state: &mut VariableCtx = &mut self.0.lock().unwrap();
        for f in &variable_state.fingerprints {
            if pc == f.func_addr {
                println!("New fingerprint with stack {:08x}", sp);
                
                let dat: DataType = match f.data_type {
                    DataType::Ptr {..} => {
                        DataType::Ptr {
                            val: 0,
                            addr: sp - f.stack_offset,
                            data: Vec::new(),
                        }
                    },
                    DataType::Int {..} => {
                        DataType::Int {
                            val: 0xffffffff,
                            addr: sp - f.stack_offset,
                        }
                    },
                    _ => panic!("Trying to insert unhandled data type")
                };
                variable_state.vars.insert(sp - f.stack_offset, dat); 
            }  
        }
    }

    pub fn process_memory_access(&self, access: &SimpleAccess) {
        let mut pid = self.0.lock().unwrap();
        match pid.vars.get_mut(&access.address) {
            Some(node) => {
                println!("updating node {:?}", node);
                node.update(access);
                println!("finished updating node {:?}", node);
            },
            None => ()
        }
    }
}

pub struct PidContext(
    pub PidTracerCtx,
    pub PidBacktraceCtx,
    pub Mutex<UnixStream>,
    pub PidVariableCtx,
);

impl Serialize for PidContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let backtrace_state: &mut BacktraceState = &mut self.1.0.lock().unwrap();
        let var_state: &mut VariableCtx = &mut self.3.0.lock().unwrap();
        let mut state = serializer.serialize_struct("Backtrace", 3)?;
        let _ = state.serialize_field("backtrace", &backtrace_state.backtrace);
        let _ = state.serialize_field("heap", &self.0.0);
        let _ = state.serialize_field("vars", &var_state.vars);
        state.end()
    }
}

impl PidContext {
    pub fn log_event(&self) {
        const POISON: u32 = 0x36afb081;
        println!("sending");
        
        {
            let pid = &mut self.0.0.lock().unwrap();
            if !pid.last_update.is_none() {
                match pid.last_update.as_ref().unwrap().operation {
                    SinkAction::Sbrk => {
                        return
                    },
                    _ => ()
                }
            }
        }
        // let backtrace_state = &mut self.1.lock().unwrap();
        let data = serde_json::to_string_pretty(
                    &self)
                    // &backtrace_state
                    // .backtrace)
                    .unwrap() + "\n";
        // packet format
        // POISON value as a u32 integer (4 bytes) followed by the schema length
        // for every possible schema (this must be synchronized b/w cannoli and
        // the receiving client). All lengths are little endian.
        //
        // schema lengths will be null for any event type that was not generated
        // by the current event.
        //
        // The current version of cannoli only generates 1 schema, so there is
        // only 1 length value and one data blob following the header.
        //
        //       u32            u32             u32       ...       u32
        // .-------------.---------------.--------------.-...-.--------------.
        // |    POISON   |  Schema 1 len | Schema 2 len | ... | Schema n len |
        // `-------------`---------------`--------------`-...-`--------------`
        //
        //   Schema x len    Schema y len
        // .---------------.---------------.-----...------.
        // | Schema x data | Schema y data |     ...      |
        // `---------------`---------------`-----...------`

        let mut header: Vec<u8> = Vec::new();
        header.extend_from_slice(&(POISON).to_le_bytes());
        header.extend_from_slice(&(data.len() as u32).to_le_bytes());
        let stream = &mut self.2.lock().unwrap();
        let _ = stream.write(&header);
        let _ = stream.write(&data.as_bytes());
        println!("sent {}", data);
    }
}
