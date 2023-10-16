//! An example user of Cannoli which collects register traces

#![feature(array_chunks)]

use std::sync::{Arc, Mutex};
use cannoli::{Cannoli, create_cannoli, SyscallParam, HostSyscallNum};
use std::os::unix::net::UnixStream;
use std::collections::HashMap;
use multimap::MultiMap;

mod arch;
mod tracer;
mod sinks;

use crate::tracer::*;
use crate::sinks::*;
use crate::utils::backtrace::{
    BacktraceState,
    PidBacktraceCtx,
};

use crate::arch::{
    get_return_address,
};

mod context;

pub use crate::context::{
    TidContext,
    PidContext,
    PidVariableCtx,
    VariableCtx,
    Fingerprint,
    DataType
};

mod utils;

pub use crate::utils::SymOff;

enum Trace {
    /// Branch execution
    Branch {
        branch: bool,
        pc: SymOff,
        ra: u64,
        state: RegState,
    },

    /// memory access
    MemAccess(SimpleAccess),

    /// mmap call
    Mmap(MemRegion),

    /// hit a sink function
    SinkHit(AllocationSink, RegState),
}

struct Mvp;

impl Cannoli for Mvp {
    type Trace = Trace;

    type TidContext = TidContext;
    type PidContext = PidContext;

    fn init_pid(c: &cannoli::ClientInfo) ->
        Arc<Self::PidContext> {
        let mut path: String = String::from("/tmp/nav_");
        path.push_str(&c.ppid.to_string());

        println!("Trying Connection to: {}", c.ppid);
        let stream = match UnixStream::connect(path) {
            Ok(s) => s,
            Err(_) => panic!("Error, cannot connect to {}", c.ppid),
        };
        println!("Connected to: {}", c.ppid);

        let state = TracerState {
            heap: None,
            allocations: MultiMap::new(),
            active_returns: MultiMap::new(),
            mappings: Vec::new(),
            output: Vec::new(),
            last_update: None,
            big_endian: c.big_endian,
            bitness: c.arch.bitness(),
        };

        // define sink in main for multiple variables

        let mut fingerprints: Vec<Fingerprint> = Vec::new();

        // push uname and group pointers
        let main_addr: u64 = 0x0400ecc;
        fingerprints.push(Fingerprint {
            func_addr: main_addr,
            stack_offset: 0x14,
            data_type: DataType::Ptr {
                addr: 0,
                val: 0,
                data: Vec::new(),
            },
        });


        fingerprints.push(Fingerprint {
            func_addr: main_addr,
            stack_offset: 0x18,
            data_type: DataType::Ptr {
                addr: 0,
                val: 0,
                data: Vec::new(),
            },
        });

        fingerprints.push(Fingerprint {
            func_addr: main_addr,
            stack_offset: 0x10,
            data_type: DataType::Int {
                addr: 0,
                val: 0,
            },
        });

        Arc::new(PidContext(
            PidTracerCtx(Mutex::new(state)),
            PidBacktraceCtx(Mutex::new(BacktraceState::default())),
            Mutex::new(stream),
            PidVariableCtx(
                Mutex::new(VariableCtx { fingerprints, vars: MultiMap::new() }),
            ),
        ),
        )
    }

    /// Load the symbol table
    fn init_tid(_pid: &Self::PidContext,
            ci: &cannoli::ClientInfo) -> (Self, Self::TidContext) {
        // Symbols
        let mut symbols = Vec::new();
        let mut alloc_symbols = Vec::new();
        let mut free_symbol: (u64, &'static str) = (0, "");
        let mut main_symbol: (u64, &'static str) = (0, "");
        let mut log_symbols = Vec::new();

        // Load the symbol file up and leak it so all the lifetimes are static
        let data = std::fs::read_to_string("symbols.txt").unwrap();
        let data = Box::leak(data.into_boxed_str());

        // Parse each line into an address and symbol
        for line in data.lines() {
            let chunk = line.splitn(3, ' ').collect::<Vec<_>>();

            let addr = u64::from_str_radix(chunk[0], 16).unwrap();
            let sym  = chunk[2];
            symbols.push((addr, sym));
            if [
                    "fgets",
                    "__isoc99_fscanf",
                    "menu"
                ]
                .contains(&sym) {
                log_symbols.push((addr, sym));
            }
            if [
                    "__libc_malloc",
                    // "__libc_calloc",
                    // "__libc_realloc",
                    "__sbrk",
                ]
                .contains(&sym) {
                alloc_symbols.push((addr, sym));
            }
            if sym == "__libc_free" { free_symbol = (addr, sym); }
            if sym == "main" { main_symbol = (addr ,sym); }
        }

        // Sort the symbols by address
        symbols.sort_by_key(|x| x.0);

        let mut sinks: HashMap<u64, AllocationSink> = HashMap::new();

        // init_sinks_from_file(sink_file.cloned(), bin_file.cloned(), &mut sinks);

        for pair in alloc_symbols {
            // println!("Trying to add symbol {:?}", pair);
            AllocationSink::add_new_from_sym(&mut sinks, pair.1, pair.0);
        }
        AllocationSink::add_new_from_sym(&mut sinks,
                                       free_symbol.1, free_symbol.0);

        AllocationSink::add_new_from_sym(&mut sinks,
                                       main_symbol.1, main_symbol.0);

        // add logging sinks
        for pair in log_symbols {
            AllocationSink::add_new_from_sym(&mut sinks, pair.1, pair.0);
        }

        println!("{:?}", sinks);
        let tid = TidTracerCtx {
            sinks: sinks,
            arch: ci.arch,
            big_endian: ci.big_endian,
        };

        (Self, TidContext {
            symbols,
            tracer_ctx: tid,
            bitness: ci.arch.bitness(),
        })
    }

    /// Trace fired on a read memory access. If the read falls within a tracked
    /// allocation boundry, log it to the trace.
    // fn read(_pid: &Self::PidContext, _tid: &Self::TidContext,
    //         pc: u64, addr: u64, val: u64, sz: u8,
    //         trace: &mut Vec<Self::Trace>) {
    //     trace.push(
    //         Trace::MemAccess(SimpleAccess {
    //             pc: pc,
    //             address: addr,
    //             size: sz,
    //             value: val,
    //             op: MemoryOperation::Read,
    //         })
    //     );
    // }

    /// Trace fired on a memory write access. If the write falls within a
    /// tracked allocation boundry, log it to the trace.
    fn write(_pid: &Self::PidContext, _tid: &Self::TidContext,
             pc: u64, addr: u64, val: u64, sz: u8,
             trace: &mut Vec<Self::Trace>) {
        trace.push(
            Trace::MemAccess(SimpleAccess {
                pc: pc,
                address: addr,
                size: sz,
                value: val,
                op: MemoryOperation::Write,
            })
        );
    }

    /// Cature syscalls
    fn syscall(_pid: &Self::PidContext, _tid: &Self::TidContext,
             num: HostSyscallNum, ret: SyscallParam, args: &Vec<SyscallParam>,
             trace: &mut Vec<Self::Trace>) {
        match num {
            HostSyscallNum::Read => {
                let read_size: usize;
                match ret {
                    SyscallParam::Int{ val } => {
                        read_size = val as usize;
                    },
                    _ => panic!("Wrong type {:?} for read ret", ret),
                }
                match &args[1] {
                    SyscallParam::RawPtr{ addr, data } => {
                       let mut bytes_read: usize = 0;
                        for i in 0..(read_size / 4) as usize {
                            trace.push(Trace::MemAccess(
                                SimpleAccess {
                                    pc: 0u64,
                                    address: *addr + (i * 4) as u64,
                                    value: u32::from_le_bytes(
                                        data[(i*4)..(i+1)*4]
                                            .try_into()
                                            .unwrap()) as u64,
                                    size: 4u8,
                                    op: MemoryOperation::Write,
                                }
                            ));
                            bytes_read += 4;
                        }
                        // handle leftover bytes
                        if read_size % 4 != 0 {
                            let mut leftover: Vec<u8> =
                                data[bytes_read..].to_vec();
                            // append to vector to allow conversion
                            for _ in 0..(4 - (data.len() % 4)) {
                                leftover.push(0u8);
                            }

                            trace.push(Trace::MemAccess(
                                SimpleAccess {
                                    pc: 0u64,
                                    address: *addr + bytes_read as u64,
                                    value: u32::from_le_bytes(
                                       leftover
                                            .try_into()
                                            .unwrap()) as u64,
                                    size: 4u8,
                                    op: MemoryOperation::Write,
                                }
                            ));
                        }
                    },
                    _ => panic!("Wrong type {:?} for second read param",
                        args[1]),
                }
            },
            _ => ()
        }
    }

    fn mmap(_pid: &Self::PidContext, _tid: &Self::TidContext,
             base: u64, len: u64, _anon: bool,
             _read: bool, _write: bool, _exec: bool, _path: &str, _offset: u64,
             trace: &mut Vec<Self::Trace>) {
        let region = MemRegion { start: base, end: base+len };
        trace.push(Trace::Mmap(region));
    }

    fn branch(_pid: &Self::PidContext, tid: &Self::TidContext,
            pc: u64, branch: bool, regs: &[u8], trace: &mut Vec<Self::Trace>) {

        // Get return instruction pointer, architecture specific
        let ra = get_return_address(regs);

        // Resolve the symbol at the PC
        let state = RegState::new(pc, regs);
        let pc_sym = tid.resolve(pc);

        // Push the inst event
        trace.push(Trace::Branch {
            pc: pc_sym,
            ra: ra as u64,
            branch,
            state
        });

        // push a sink event if entering a sink function
        let reg_state = RegState::new(pc, regs);
        match tid.tracer_ctx.sinks.get(&pc) {
            Some(sink) =>
                trace.push(Trace::SinkHit(sink.clone(), reg_state)),
            None => (),
        }

    }

    fn trace(&mut self, pid: &Self::PidContext, tid: &Self::TidContext,
             trace: &[Self::Trace]) {
        for ent in trace {
            match ent {
                Trace::Branch { pc, ra, branch, state } => {
                    // handle backtrace and heap return operations if flag is set
                    if pid.1.is_branch() {

                        // check if this is a branch associated with an active ret
                        // if matched then this is a new allocation. Log event
                        // must log here instead of the SinkHit event because 
                        // the allocation is not pushed until *alloc's return
                        if pid.0.process_sink_return_action(state) {
                            // pid.log_event();
                        }

                        // check if this instruction is the entry to a func
                        let sym = tid.resolve(pc.addr());
                        if sym.is_entry() {
                            // push symbol to backtrace stack
                            pid.1.push_backtrace(sym.symbol(),pc.addr());
                            // push link register to the return stack
                            pid.1.push_return_stack(*ra);
                            // pid.log_event();
                        }
                        // else check if this instruction matches the expected
                        // return address (last val pushed to the ret stack)
                        // need to check in loop to handle branch w/o link
                        loop {
                            if pid.1.is_return(pc.addr()) {
                                // pop two stacks
                                pid.1.pop_backtrace();
                                pid.1.pop_return_stack();

                                // log new backtrace state
                                // pid.log_event();
                            }
                            else { break }
                        }

                        // clear flag
                        pid.1.unset_branch_flag();
                    }
                    if *branch {
                        pid.1.set_branch_flag();
                    }
                },
                Trace::MemAccess(access) => {
                    // match access.op {
                    //     MemoryOperation::Write =>
                    //         println!("WRITE at {:08x} with value {:08x}",
                    //             access.address, access.value),
                    //     MemoryOperation::Read =>
                    //         println!("READ  at {:08x} with value {:08x}",
                    //             access.address, access.value),
                    //     _ => (),
                    // }
                    pid.0.process_memory_access(access, true);
                    // record new value in fingerprinted addresses, if relevant
                    pid.3.process_memory_access(access);
                },
                Trace::Mmap(region) =>
                    pid.0.process_mmap(&region),
                Trace::SinkHit(sink, state) => {
                    pid.0.process_sink_call_action(&sink.clone(), &state);
                    match sink.operation {
                        // on entry to main, fingerprint variable addresses
                        SinkAction::Main => {
                            if pid.3.is_fingerprinted(state.pc){
                                pid.3.fingerprint(state.pc, state.sp);
                            }
                            eprintln!("finished fingerprinting {:?}", pid.3);
                        },
                        // if this a free operation then log the event
                        SinkAction::Free => (), // pid.log_event(),
                        SinkAction::Log => {
                            println!("Logging from log function");
                            pid.log_event()
                        },
                        _ => ()
                    }
                },
            }
        }
    }

}

fn main() {
    println!("Starting client");
    create_cannoli::<Mvp>(4).unwrap();
}
