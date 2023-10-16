use crate::tracer::RegState;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SinkDefinition {
    address: u64,
    name_exact: String,
    name_contains: String,
    operation: String,
    args: Vec<String>,
    returns: String,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub enum SinkAction {
    Allocate,
    ReAllocate,
    Sbrk,
    Free,
    Unk,
    Main,
    Log,
}

impl SinkAction {
    /// Convert a String into a SinkAction 
    fn _from_string(t: String) -> SinkAction {
        match t.as_str() {
            "Allocate"   => SinkAction::Allocate,
            "ReAllocate" => SinkAction::ReAllocate,
            "Free"       => SinkAction::Free,
            "Main"       => SinkAction::Main,
            "Log"        => SinkAction::Log,
            _            => SinkAction::Unk,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum VarType {
    Pointer,
    Size,
    Num,
    Void,
    Unk,
}

impl VarType {
    /// Convert a String into a VarType
    fn _from_string(t: String) -> VarType {
        match t.as_str() {
            "Pointer" => VarType::Pointer,
            "Size"    => VarType::Size,
            "Num"     => VarType::Num,
            "Void"    => VarType::Void,
            _         => VarType::Unk,
        }
    }
}

/// Defines the internal structure of an allocation sink
///
/// address: Address of the given sink function
/// operation: Specific operation performed by the sink function
/// args: An ordered vector of the arguments expected to be passed to the sink
///       funciton
/// names: Vector of strings used as names for the functions. eg malloc, 
///        _malloc, __libc_malloc
/// returns: Type returned by the sink function
#[derive(Clone, Debug)]
pub struct AllocationSink {
    pub address: u64,
    pub operation: SinkAction,
    pub args: Vec<VarType>,
    pub names: Vec<String>,
    pub returns: VarType,
}

pub fn _init_sinks_from_file(sink_file: Option<String>,
                            binary_file: Option<String>,
                            _sinks: &mut HashMap<u64, AllocationSink>) {
    if sink_file.is_none() {
        panic!("No sink file provided.");
    }

    if ! binary_file.is_none() {
        let output = std::process::Command::new("nm")
            .arg(binary_file.unwrap())
            .output()
            .expect("Failed to execute `nm`");
        let data = Box::leak(output.stdout.into_boxed_slice());
        let data = std::str::from_utf8(data)
            .expect("Failed to decode symbols as UTF-8");
        for line in data.lines() {
            if !line.contains(" t ") && !line.contains(" T ") {
                continue;
            }

            let chunk = line.splitn(3, ' ').collect::<Vec<_>>();

            let _ = u64::from_str_radix(chunk[0], 16).unwrap();
            let _ = chunk[2];
            // ADD SYMBOL HERE
        }
    }
}

impl AllocationSink {
    pub fn add_new_from_sym(sinks: &mut HashMap<u64, AllocationSink>,
                            sym: &'static str,
                            addr: u64) {
        let mut names = match sinks.get(&addr) {
            Some(s) => s.names.to_vec(),
            None => Vec::new(),
        };

        let info = if sym.contains("libc_malloc") {
            Some((SinkAction::Allocate,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else if sym.contains("libc_calloc") {
            Some((SinkAction::Allocate,
                  vec![VarType::Size, VarType::Num],
                  VarType::Pointer))
        } else if sym.contains("libc_free") {
            Some((SinkAction::Free,
                  vec![VarType::Pointer],
                  VarType::Void))
        } else if sym.contains("sbrk") {
            Some((SinkAction::Sbrk,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else if sym.contains("main") {
            Some((SinkAction::Main,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else if sym.contains("menu") {
            Some((SinkAction::Log,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else if sym.contains("fgets") {
            Some((SinkAction::Log,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else if sym.contains("fscanf") {
            Some((SinkAction::Log,
                  vec![VarType::Size],
                  VarType::Pointer))
        } else { None };

        if info.is_none() {
            return ();
        }

        names.push(sym.to_string());

        let sink = AllocationSink {
            address   : addr,
            operation : info.as_ref().unwrap().0,
            args      : info.as_ref().unwrap().1.to_vec(),
            names     : names,
            returns   : info.as_ref().unwrap().2,
        };

        sinks.insert(addr, sink);
    }

    /// Calculate and return the allocation size of a given sink function and
    /// the current register state
    pub fn get_allocation_size(&self, state: &RegState) -> usize {
        let mut size = 0;
        let mut num = 1;
        let emu_args = [
            state.arg0,
            state.arg1,
            state.arg2,
            state.arg3,
        ];

        if self.args.len() > emu_args.len() {
            panic!("Potentially malformed sink or emulator args #.");
        }

        for i in 0..self.args.len() {
            match self.args[i] {
                VarType::Size => size = emu_args[i],
                VarType::Num => num = emu_args[i],
                _ => continue,
            }
        }
        let allocation_size = size * num;
        allocation_size.try_into().unwrap()
    }
}
