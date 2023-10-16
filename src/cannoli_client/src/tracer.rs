use crate::sinks::{SinkAction, AllocationSink, VarType};
use serde::{Deserialize, Serialize};
use serde::ser::{Serializer, SerializeStruct};
use entropy::{metric_entropy};

use crate::arch::{
    get_args,
    get_return_address,
    get_return_value,
    get_stack_pointer
};

use cannoli::Architecture;
use core::arch::x86_64::_rdtsc;
use std::collections::HashMap;
use multimap::MultiMap;
use std::sync::Mutex;

type Address = Option<u64>;

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug)]
pub struct Allocation {
    pub id: u64,
    pub pc: u64,
    pub start: Address,
    pub end: Address,
    pub size: usize,
}

impl Allocation {
    /// Check if the Allocation falls neatly within a range
    fn in_range(&self, region: &MemRegion) -> bool {
        match (self.start, self.end) {
            // If allocation has start and end
            (Some(start), Some(end)) => {
                if start >= region.start && end < region.end {
                    return true;
                }
                return false;
            },
            (_, _) => false,
        }
    }

    fn contains_address(&self, addr: u64) -> bool {
        match (self.start, self.end) {
            // If allocation has start and end
            (Some(start), Some(end)) => {
                if addr >= start && addr < end {
                    return true;
                }
                return false;
            },
            (_, _) => false,
        }
    }
}

impl std::fmt::Display for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]",
               match self.start {
                   Some(a) => format!("{:#x}", a),
                   None => "[none]".to_string()
               },
               match self.end {
                   Some(a) => format!("{:#x}", a),
                   None => "[none]".to_string()
               })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AllocMeta {
    pub alloc: Allocation,
    pub operation: SinkAction,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct AllocLatest {
    pub start: u64,
    pub allocation: Vec<Allocation>,
    pub operation: SinkAction,
}

#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct MemRegion {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Debug)]
pub struct Heap {
    pub mem_region: MemRegion,
    pub memory: Vec<u8>,
    pub big_endian: bool,
    pub bitness: u8,
    pub entropy: f32,
}

impl Serialize for Heap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let mut state = serializer.serialize_struct("Heap", 2)?;
        let _ = state.serialize_field("bounds", &self.mem_region);
        let _ = state.serialize_field("entropy", &self.entropy);
        state.end()
    }
}

impl Heap {
    fn start(&self) -> u64 {
        self.mem_region.start
    }
    fn end(&self) -> u64 {
        self.mem_region.end
    }
    fn update(&mut self, val: u64, write_size: u64, addr: u64) {
        // assert that the write is contained within the heap
        if ((addr + write_size) > self.end()) ||
            addr < self.start() {
            panic!("Write at {:#x} of {:#x} bytes for heap {:?}",
                addr,
                write_size,
                self);
        }

        let start_idx: usize = (addr - self.start()) as usize;
        let end_idx: usize = start_idx + write_size as usize;

        if self.big_endian {
            for i in start_idx..end_idx {
                self.memory[i] = ((val >> (end_idx - i - 1) * 8) & 0xff) as u8;
                // println!("writing {:#x} at {:x}",
                //     ((val >> (end_idx - i - 1) * 8) & 0xff) as u8,
                //     i + self.start() as usize,
                // );
            }
        } else {
            for i in start_idx..end_idx {
                self.memory[i] = ((val >> (i - start_idx) * 8) & 0xff) as u8;
                // println!("writing {:#x} at {:x}",
                //     ((val >> (i - start_idx) * 8) & 0xff) as u8,
                //     i + self.start() as usize,
                // );
            }
        }
        // update entropy
        self.entropy = metric_entropy(&self.memory);
    }

    // hexdump based on bitness and endianness
    fn hexdump(&self) -> String {
        let start: usize = self.start() as usize;
        let long_size: usize = self.bitness as usize / 8usize;
        let mut dump = String::new();

        for i in 0..(self.end() as usize - start) / 16 {
            let line: String;
            if long_size == 8 { line = format!("0x{:016x}:", start + i * 16); }
            else { line = format!("0x{:08x}:", start + i * 16); }
            dump.push_str(&line);
            let mut line = String::new();
            for j in 0..16 {
                // print space between each usize bytes
                if j % long_size == 0 { line.push(' '); }
                if self.big_endian {
                    line.push_str(
                        &format!("{:02x}", self.memory[i * 16 + j as usize])
                    );
                } else {
                    // print little endian dump similar to gdb, where dwords /
                    // qwords have LSB right-aligned (essentially big-endian)
                    line.push_str(&format!("{:02x}",
                        self.memory[
                            i * 16usize +
                            (j as usize / long_size) * long_size +
                            (long_size - 1 - (j as usize % long_size) as usize)
                        ]
                    ));
                }
            }
            dump.push_str(&line);
            dump.push('\n');
        }
        dump
    }
}

impl std::fmt::Display for Heap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        _ = write!(f, "Heap: {:?}, Entropy: {}\n", self.mem_region, self.entropy);
        _ = write!(f, "{}", self.hexdump());
        _ = write!(f, "\n");
        Ok(())
    }
}

#[derive(Debug)]
pub struct TracerState {
    pub heap: Option<Heap>,
    pub allocations: MultiMap<u64, Allocation>,
    pub active_returns: MultiMap<u64, AllocMeta>,
    pub mappings: Vec<MemRegion>,
    pub output: Vec<String>,
    pub last_update: Option<AllocLatest>,
    pub big_endian: bool,
    pub bitness: u8,
}

impl Serialize for TracerState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let mut state = serializer.serialize_struct("State", 3)?;
        let _ = state.serialize_field("meta", &self.heap);
        let _ = state.serialize_field("allocations", &self.allocations);
        let _ = state.serialize_field("last_heap_update", &self.last_update);
        state.end()
    }
}

/// Tuple struct containing the Mutex we put in an Arc for the PidTracerContext.
pub struct PidTracerCtx(pub Mutex<TracerState>);

pub struct TidTracerCtx {
    pub sinks: HashMap<u64, AllocationSink>,
    pub arch: Architecture,
    pub big_endian: bool,
}

// pub struct AllocationTracer;

#[derive(Copy, Clone, Debug)]
pub enum MemoryOperation {
    Read,
    Write,
}

impl MemoryOperation {
    pub fn as_str(&self) -> &str {
        match self {
            MemoryOperation::Read => "READ",
            MemoryOperation::Write => "WRITE",
        }
    }
}

/// Define a memory access of:
///     An address being accessed
///     Which allocation is being accessed
///     If the operation is a Read or Write
///     The size of the access
#[derive(Copy, Clone, Debug)]
pub struct MemoryAccess {
    pub address: Address,
    pub allocation: Option<Allocation>,
    pub operation: MemoryOperation,
    pub size: u8,
    pub pc: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleAccess {
    pub pc: u64,
    pub address: u64,
    pub size: u8,
    pub value: u64,
    pub op: MemoryOperation,
}

#[derive(Debug)]
pub struct RegState {
    pub pc: u64,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub ret_addr: u64,
    pub ret_val: u64,
    pub sp: u64,
}

impl RegState {
    pub fn new(pc: u64, regs: &[u8]) -> RegState {
        let args = get_args(regs);
        RegState {
            pc: pc,
            arg0: args[0] as u64,
            arg1: args[1] as u64,
            arg2: args[2] as u64,
            arg3: args[3] as u64,
            ret_addr: get_return_address(regs) as u64,
            ret_val: get_return_value(regs) as u64,
            sp: get_stack_pointer(regs) as u64,
        }
    }
}

// #[derive(Debug)]
// pub enum TraceAction {
//     Mmap(MemRegion),
//     SinkHit(AllocationSink, RegState),
//     MemAccess(SimpleAccess),
//     Inst(RegState),
// }

#[inline]
fn get_time() -> u64 {
    unsafe { _rdtsc() }
}

/// Remove a single AllocMeta from the MultiMap if there are multiple instances
/// of AllocMeta with the same key. Else, remove the key from the map.
fn remove_from_returns(active_returns: &mut MultiMap<u64, AllocMeta>,
                       pc: u64) {
    let v = active_returns.get_vec_mut(&pc).unwrap();

    // Check if there are multiple allocations from a given
    // PC return
    if v.len() > 1 {
        // Find the allocation with the lowest ID
        let res = v.iter()
                   .enumerate()
                   .min_by_key(|(_, alloc_meta)| alloc_meta.alloc.id)
                   .map(|(i, _)| i).unwrap();
        // Remove it from the vec
        v.remove(res);
    } else {
        // Else, remove the key from the map
        active_returns.remove(&pc).unwrap();
    }
}

impl PidTracerCtx {
    fn sink_free_call(&self, state: &RegState) -> bool {
        let addr: u64 = state.arg0;

        let pid = &mut self.0.lock().unwrap();

        // Remove the allocation from our active allocations
        match pid.allocations.remove(&addr) {
            // If we've removed alloc[s], return the 1st as a trace in the
            // event of mutliple of a given key
            Some(allocs) => {
                if allocs.len() > 1 {
                    println!(
                        "[WARN] free({addr:#x}) may free multiple allocations."
                    );
                }

                /*pid.output.push(format!("{:#010x}: Free {}",
                                        state.pc,
                                        allocs[0]));
                */
                let start = allocs.first().unwrap().start.unwrap();
                pid.last_update = Some(
                    AllocLatest {
                        allocation: allocs,
                        start,
                        operation: SinkAction::Free,
                    }
                );
                true
            },
            // We haven't removed anything so we return the a placeholder
            // allocation which only contains a start address
            None => {
                /* pid.output.push(format!("{:#010x}: Free {} !!! [UNTRACKED]",
                                        state.pc,
                                        addr));
                */
                return false
            },
        }
    }

    fn sink_alloc_call(&self, sink: &AllocationSink, state: &RegState) -> bool {
        // Get the return address so that we can capture the returned
        // address to the allocation
        let ret = state.ret_addr;

        // Get allocation size based on argument definition in the sink
        let alloc_size = sink.get_allocation_size(state);

        // Build our new placeholder allocation
        let alloc = Allocation {
            id: get_time(),
            pc: state.ret_addr,
            start: None,
            end: None,
            size: alloc_size,
        };

        let meta = AllocMeta {
            alloc: alloc,
            operation: sink.operation,
        };

        // Insert the sink & temp allocation into our map
        self.0.lock()
            .unwrap()
            .active_returns
            .insert(ret.into(), meta);
        true
    }

    pub fn process_sink_call_action(&self,
                                    sink: &AllocationSink,
                                    state: &RegState) -> bool {
        match sink.operation {
            SinkAction::Allocate => self.sink_alloc_call(sink, state),
            SinkAction::Sbrk => self.sink_alloc_call(sink, state),
            SinkAction::Free => self.sink_free_call(state),
            SinkAction::ReAllocate => panic!("ReAlloc sinks not supported."),
            SinkAction::Main => false,
            SinkAction::Log => false,
            SinkAction::Unk => panic!("Unknown sink type."),
        }
    }

    pub fn is_matched_return(&self, state: &RegState) -> bool {
        let pid = &mut self.0.lock().unwrap();
        let alloc_metadata =
            match pid.active_returns.get(&state.pc) {
                Some(meta) => meta,
                None => return false,
        };

        match alloc_metadata.operation {
            SinkAction::Allocate => {
                let start: u64 = state.ret_val;

                // Check if allocation has failed
                if start == 0 {
                    return false
                }
                return true
            },
            _ => return false,
        }
    }

    pub fn process_sink_return_action(&self, state: &RegState) -> bool {
        let pid = &mut self.0.lock().unwrap();
        let alloc_metadata =
            match pid.active_returns.get(&state.pc) {
                Some(meta) => meta,
                None => return false,
        };

        match alloc_metadata.operation {
            SinkAction::Allocate => {
                let mut alloc = alloc_metadata.alloc;
                let start: u64 = state.ret_val;

                // Check if allocation has failed
                if start == 0 {
                    return false
                }

                let end = start + alloc.size as u64;
                alloc.start = Some(start);
                alloc.end = Some(end);

                let mut region: Option<Heap> = None;
                if pid.heap.is_none() {
                    for mapping in &pid.mappings {
                        if alloc.in_range(mapping) {
                            let len = mapping.end - mapping.start;
                            region = Some(
                                        Heap {
                                            mem_region: mapping.clone(),
                                            memory: vec![0u8; len as usize],
                                            big_endian: pid.big_endian,
                                            bitness: pid.bitness,
                                            entropy: 0f32,
                                        });
                        }
                    }
                    if region.is_none() {
                        println!(
                            "[+] Failed to find mapping for allocaiton @ {:#x}",
                            start);
                    } else {
                        pid.heap = region;
                    }
                }

                // Add the allocation to our map of active allocations
                pid.allocations.insert(start, alloc.clone());
                remove_from_returns(&mut pid.active_returns, state.pc);

                // update lastest sink return and return the vec of vals
                let v = pid.allocations.get_vec(&start).unwrap().clone();
                pid.last_update = Some(
                    AllocLatest {
                        start: start,
                        operation: SinkAction::Allocate,
                        allocation: v,
                    }
                );

                // pid.output.push(format!("{:#010x}: Alloc {}", state.pc, alloc));
                return true
            },
            SinkAction::Sbrk => {
                // void* sbrk(usize size)
                // AllocMeta will include an Allocation with size: `size` as
                // requested by funtion call. This size is the expansion.
                // if successful, return value is the old end of the heap, and
                // program can assume that the requested memory was allocated.
                //
                // note, brk() reserves page-aligned memory but sbrk does not
                // know about the extra memory left on the page, only the end
                // as requested from the previous call. So this abstraction
                // means we do not have to care where the end of the heap *page*
                // is, just the end of the requested heap memory
                //
                // e.g., if heap ends at 0xa0010, brk already reserved, as an
                // example, the page 0xa0000-0xa01000 from the last sbrk req.
                // If sbrk reqs 0x10 more bytes, brk returns without error and
                // without mapping more memory. Sbrk returns success with
                // return value 0xa0010 and the new end of heap as 0xa0020
                let mut alloc = alloc_metadata.alloc;
                let old_heap_end: u64 = state.ret_val;
                // check if allocation failed
                if old_heap_end as i64 == -1 {
                    return false
                }

                // return val points to the beginning of newly allocated memory
                let end = old_heap_end + alloc.size as u64;
                alloc.start = Some(old_heap_end);
                alloc.end = Some(end);

                if pid.heap.is_none() {
                    // sbrk call for first allocation, set the heap.
                    //
                    // assert that the returned pointer (old heap end) is page-
                    // aligned, because first call to sbrk should return the
                    // the beginning of the heap.
                    assert_eq!(old_heap_end, old_heap_end & !0xfff);
                    pid.heap =
                        Some(
                            Heap {
                                mem_region: MemRegion {
                                    start: old_heap_end,
                                    end: end,
                                },
                                memory: vec![0u8; (end-old_heap_end) as usize],
                                big_endian: pid.big_endian,
                                bitness: pid.bitness,
                                entropy: 0f32,
                            }
                        );
                } else {
                    // update heap range and entropy
                    let mut h = pid.heap.as_mut().unwrap();
                    let start = h.start();
                    h.mem_region.end = end;
                    h.memory.resize((end - start) as usize, 0);
                    h.entropy = metric_entropy(&h.memory);
                }

                remove_from_returns(&mut pid.active_returns, state.pc);

                // update lastest sink return
                let mut v: Vec<Allocation> = Vec::new();
                v.push(alloc.clone());
                pid.last_update = Some(
                    AllocLatest {
                        start: old_heap_end,
                        operation: SinkAction::Sbrk,
                        allocation: v,
                    }
                );
                return true
            },
            _ => panic!("Sink action is not implemented."),
        }
    }

    pub fn process_memory_access(&self, access: &SimpleAccess, memoize: bool) {
        let mut found_alloc: Option<Allocation> = None;
        let mut pid = self.0.lock().unwrap();

        // Search for address in active allocations
        for (_, alloc) in pid.allocations.iter() {
            if alloc.contains_address(access.address) {
                found_alloc = Some(alloc.clone())
            }
        };
        match found_alloc {
            Some(alloc) => {
                let _o = format!("{:#010x}: {:>5}{:<2} {:#x}+{:#x} {{{:#x}}}",
                                access.pc,
                                access.op.as_str(),
                                access.size,
                                alloc.start.unwrap(),
                                access.address - alloc.start.unwrap(),
                                access.value);
                // pid.output.push(o)

                // update heap if write and if memoization is required
                if memoize {
                    match access.op {
                        // update heap memory if write
                        MemoryOperation::Write =>  {
                            if !pid.heap.is_none() {
                                pid.heap.as_mut().unwrap().update(access.value,
                                    access.size as u64,
                                    access.address);
                            } else {
                                panic!("No heap found for write at {:08x}",
                                    access.address);
                            }
                        },
                        _ => (),
                    }
                }
            },
            None => {
                match &mut pid.heap {
                    Some(heap) =>
                        // check if memory access is within active heap
                        if access.address >= heap.start() &&
                                access.address < heap.end() {
                            // let o = format!(
                            //     "{:#010x}: {:>5}{:<2} {:#x} {{{:#x}}}",
                            //     access.pc,
                            //     access.op.as_str(),
                            //     access.size,
                            //     access.address,
                            //     access.value
                            // );
                            // pid.output.push(o);

                            // update heap if write and if memoization required
                            if memoize {
                                match access.op {
                                    MemoryOperation::Write => {
                                        heap.update(access.value,
                                             access.size as u64,
                                             access.address);
                                    },
                                    _ => (),
                                }
                            }
                        },
                    None => (),
                }
            },
        }
    }

    pub fn process_mmap(&self, region: &MemRegion) {
        self.0.lock().unwrap().mappings.push(region.clone());
    }
}


/// Tests
/// Heap tests
fn _test_heap_write_32_le() {
    let tracer_ctx = TracerState {
        heap: Some(Heap {
            mem_region: MemRegion { start: 0xa8000, end: 0xa8010 },
            memory: vec![0; 0x10],
            big_endian: false,
            bitness: 32,
            entropy: 0f32,
        }),
        allocations: MultiMap::new(),
        active_returns: MultiMap::new(),
        mappings: Vec::new(),
        output: Vec::new(),
        last_update: None,
        big_endian: false,
        bitness: 32,
    };
    let pid = PidTracerCtx(Mutex::new(tracer_ctx));
    let s = SimpleAccess {
        pc: 0x1000,
        address: 0xa8000,
        size: 4,
        value: 0x41424344,
        op: MemoryOperation::Write,
    };
    pid.process_memory_access(&s, true);
    assert_eq!(pid.0.lock().unwrap().heap.as_ref().unwrap().memory,
        [ 0x44, 0x43, 0x42, 0x41,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0 ]
    );

    // test display
    let disp = format!("{}",
        pid.0.lock().unwrap().heap.as_ref().unwrap().hexdump()
    );
    assert_eq!(disp, "0x000a8000: 41424344 00000000 00000000 00000000\n");
}

fn _test_heap_write_64_le() {
    let tracer_ctx = TracerState {
        heap: Some(Heap {
            mem_region: MemRegion { start: 0xa8000, end: 0xa8010 },
            memory: vec![0; 0x10],
            big_endian: false,
            bitness: 64,
            entropy: 0f32,
        }),
        allocations: MultiMap::new(),
        active_returns: MultiMap::new(),
        mappings: Vec::new(),
        output: Vec::new(),
        last_update: None,
        big_endian: false,
        bitness: 64,
    };
    let pid = PidTracerCtx(Mutex::new(tracer_ctx));
    let s = SimpleAccess {
        pc: 0x1000,
        address: 0xa8000,
        size: 8,
        value: 0x4142434445,
        op: MemoryOperation::Write,
    };
    pid.process_memory_access(&s, true);
    assert_eq!(pid.0.lock().unwrap().heap.as_ref().unwrap().memory,
        [ 0x45, 0x44, 0x43, 0x42,
        0x41, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0 ]
    );

    // test display
    let disp = format!("{}",
        pid.0.lock().unwrap().heap.as_ref().unwrap().hexdump()
    );
    assert_eq!(disp,
        "0x00000000000a8000: 0000004142434445 0000000000000000\n"
    );
}

fn _test_heap_write_32_be() {
    let tracer_ctx = TracerState {
        heap: Some(Heap {
            mem_region: MemRegion { start: 0xa8000, end: 0xa8010 },
            memory: vec![0; 0x10],
            big_endian: true,
            bitness: 32,
            entropy: 0f32,
        }),
        allocations: MultiMap::new(),
        active_returns: MultiMap::new(),
        mappings: Vec::new(),
        output: Vec::new(),
        last_update: None,
        big_endian: true,
        bitness: 32,
    };
    let pid = PidTracerCtx(Mutex::new(tracer_ctx));
    let s = SimpleAccess {
        pc: 0x1000,
        address: 0xa8000,
        size: 4,
        value: 0x41424344,
        op: MemoryOperation::Write,
    };
    pid.process_memory_access(&s, true);
    assert_eq!(pid.0.lock().unwrap().heap.as_ref().unwrap().memory,
        [0x41, 0x42, 0x43, 0x44,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0 ]
    );

    // test display
    let disp = format!("{}",
        pid.0.lock().unwrap().heap.as_ref().unwrap().hexdump()
    );
    assert_eq!(disp, "0x000a8000: 41424344 00000000 00000000 00000000\n");
}

fn _test_heap_write_64_be() {
    let tracer_ctx = TracerState {
        heap: Some(Heap {
            mem_region: MemRegion { start: 0xa8000, end: 0xa8010 },
            memory: vec![0; 0x10],
            big_endian: true,
            bitness: 64,
            entropy: 0f32,
        }),
        allocations: MultiMap::new(),
        active_returns: MultiMap::new(),
        mappings: Vec::new(),
        output: Vec::new(),
        last_update: None,
        big_endian: true,
        bitness: 64,
    };
    let pid = PidTracerCtx(Mutex::new(tracer_ctx));
    let s = SimpleAccess {
        pc: 0x1000,
        address: 0xa8000,
        size: 8,
        value: 0x4142434445,
        op: MemoryOperation::Write,
    };
    pid.process_memory_access(&s, true);
    assert_eq!(pid.0.lock().unwrap().heap.as_ref().unwrap().memory,
        [0, 0, 0, 0x41,
        0x42, 0x43, 0x44, 0x45,
        0, 0, 0, 0,
        0, 0, 0, 0 ]
    );

    // test display
    let disp = format!("{}",
        pid.0.lock().unwrap().heap.as_ref().unwrap().hexdump()
    );
    assert_eq!(disp,
        "0x00000000000a8000: 0000004142434445 0000000000000000\n"
    );
}

fn _test_sbrk() {
    let tracer_ctx = TracerState {
        heap: Some(Heap {
            mem_region: MemRegion { start: 0xa8000, end: 0xa8010 },
            memory: vec![0; 0x10],
            big_endian: false,
            bitness: 32,
            entropy: 0f32,
        }),
        allocations: MultiMap::new(),
        active_returns: MultiMap::new(),
        mappings: Vec::new(),
        output: Vec::new(),
        last_update: None,
        big_endian: false,
        bitness: 32,
    };
    let pid = PidTracerCtx(Mutex::new(tracer_ctx));
    let s = SimpleAccess {
        pc: 0x1000,
        address: 0xa8000,
        size: 4,
        value: 0x41424344,
        op: MemoryOperation::Write,
    };
    pid.process_memory_access(&s, true);

    // fake sbrk call: `sbrk(0x10);`
    let sink = AllocationSink {
        address: 0xcafebabe,
        operation: SinkAction::Sbrk,
        args: vec![VarType::Size; 1],
        names: vec!["__sbrk".to_string(); 1],
        returns: VarType::Pointer,
    };
    let regs = RegState {
        pc: 0xcafebabe,
        arg0: 0x10,
        arg1: 0x0,
        arg2: 0x0,
        arg3: 0x0,
        ret_addr: 0xdeadbeef,
        ret_val: 0x0,
        sp: 0x0,
    };

    pid.process_sink_call_action(&sink, &regs);

    // create return state to handle sbrk return
    // returned value (if successful) is the previous end of the heap
    let regs = RegState {
        pc: 0xdeadbeef,
        arg0: 0x0,
        arg1: 0x0,
        arg2: 0x0,
        arg3: 0x0,
        ret_addr: 0x0,
        ret_val: 0xa8010,
        sp: 0x0,
    };

    pid.process_sink_return_action(&regs);

    assert_eq!(pid.0.lock().unwrap().heap.as_ref().unwrap().memory,
        [ 0x44, 0x43, 0x42, 0x41,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0 ]
    );

    // test display
    let disp = format!("{}",
        pid.0.lock().unwrap().heap.as_ref().unwrap().hexdump()
    );
    assert_eq!(disp,
        "\
        0x000a8000: 41424344 00000000 00000000 00000000\n\
        0x000a8010: 00000000 00000000 00000000 00000000\n");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_heap() {
        _test_heap_write_32_le();
        _test_heap_write_64_le();
        _test_heap_write_32_be();
        _test_heap_write_64_be();
        _test_sbrk();
    }
}
