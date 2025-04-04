use std::{
    fs::File,
    io::{Seek, Write},
};

use hashbrown::HashMap;
use serde::{self, Deserialize, Serialize};

use serde_big_array::BigArray;
use crate::{events::MemoryRecord, syscalls::SyscallCode, ExecutorMode};


// 2GB memory space for the program with 32 bit address space
/// The maximum number of memory addresses that can be tracked.
pub const MAXIMUM_ADDRESSES: usize = 1 << 29;

/// The memory of the program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory(
    #[serde(with = "BigArray")]
    pub [MemoryRecord; MAXIMUM_ADDRESSES]
);

impl Default for Memory {
    fn default() -> Self {
        Self([MemoryRecord::default(); MAXIMUM_ADDRESSES])
    }
}

/// Holds data describing the current state of a program's execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct ExecutionState {
    /// The program counter.
    pub pc: u32,

    /// The shard clock keeps track of how many shards have been executed.
    pub current_shard: u32,

    /// The memory which instructions operate over. Values contain the memory value and last shard
    /// + timestamp that each memory address was accessed.
    pub memory: Box<Memory>,

    /// The global clock keeps track of how many instructions have been executed through all shards.
    pub global_clk: u64,

    /// The clock increments by 4 (possibly more in syscalls) for each instruction that has been
    /// executed in this shard.
    pub clk: u32,

    /// Uninitialized memory addresses that have a specific value they should be initialized with.
    /// `SyscallHintRead` uses this to write hint data into uninitialized memory.
    pub uninitialized_memory: HashMap<u32, u32>,

    /// A stream of input values (global to the entire program).
    pub input_stream: Vec<Vec<u8>>,

    /// A ptr to the current position in the input stream incremented by `HINT_READ` opcode.
    pub input_stream_ptr: usize,

    /// A ptr to the current position in the proof stream, incremented after verifying a proof.
    pub proof_stream_ptr: usize,

    /// A stream of public values from the program (global to entire program).
    pub public_values_stream: Vec<u8>,

    /// A ptr to the current position in the public values stream, incremented when reading from
    /// `public_values_stream`.
    pub public_values_stream_ptr: usize,

    /// Keeps track of how many times a certain syscall has been called.
    pub syscall_counts: HashMap<SyscallCode, u64>,

    // Replace HashMap-based register storage with direct arrays
    // Hot registers (x0-x7) are accessed most frequently
    pub hot_registers: [MemoryRecord; 8],
    // Cold registers (x8-x31) are accessed less frequently
    pub cold_registers: [MemoryRecord; 24],
}

impl ExecutionState {
    #[must_use]
    /// Create a new [`ExecutionState`].
    pub fn new(pc_start: u32) -> Self {
        Self {
            global_clk: 0,
            // Start at shard 1 since shard 0 is reserved for memory initialization.
            current_shard: 1,
            clk: 0,
            pc: pc_start,
            memory: Default::default(),
            uninitialized_memory: HashMap::new(),
            input_stream: Vec::new(),
            input_stream_ptr: 0,
            public_values_stream: Vec::new(),
            public_values_stream_ptr: 0,
            proof_stream_ptr: 0,
            syscall_counts: HashMap::new(),
            hot_registers: [MemoryRecord::default(); 8],
            cold_registers: [MemoryRecord::default(); 24],
        }
    }
    
    // Helper method to get a register value
    pub fn get_register(&self, reg: usize) -> &MemoryRecord {
        if reg < 8 {
            &self.hot_registers[reg]
        } else {
            &self.cold_registers[reg - 8]
        }
    }
    
    // Helper method to set a register value
    pub fn set_register(&mut self, reg: usize, record: MemoryRecord) {
        // Register x0 is hardwired to zero in RISC-V
        if reg == 0 {
            return;
        }
        
        if reg < 8 {
            self.hot_registers[reg] = record;
        } else {
            self.cold_registers[reg - 8] = record;
        }
    }
}

/// Holds data to track changes made to the runtime since a fork point.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ForkState {
    /// The `global_clk` value at the fork point.
    pub global_clk: u64,
    /// The original `clk` value at the fork point.
    pub clk: u32,
    /// The original `pc` value at the fork point.
    pub pc: u32,
    /// All memory changes since the fork point.
    pub memory_diff: HashMap<u32, Option<MemoryRecord>>,
    // /// The original memory access record at the fork point.
    // pub op_record: MemoryAccessRecord,
    // /// The original execution record at the fork point.
    // pub record: ExecutionRecord,
    /// Whether `emit_events` was enabled at the fork point.
    pub executor_mode: ExecutorMode,
}

impl ExecutionState {
    /// Save the execution state to a file.
    pub fn save(&self, file: &mut File) -> std::io::Result<()> {
        let mut writer = std::io::BufWriter::new(file);
        bincode::serialize_into(&mut writer, self).unwrap();
        writer.flush()?;
        writer.seek(std::io::SeekFrom::Start(0))?;
        Ok(())
    }
}
