use serde::{Deserialize, Serialize};

use crate::events::{
    memory::{MemoryReadRecord, MemoryWriteRecord},
    LookupId, MemoryLocalEvent,
};

/// Square Event.
///
/// This event is emitted when a square operation is performed.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SquareEvent {
    /// The lookup identifier.   
    pub lookup_id: LookupId,
    /// The shard number.
    pub shard: u32,
    /// The clock cycle.
    pub clk: u32,
    /// The pointer to the input.
    pub input_ptr: u32,
    /// The pointer to the output.
    pub output_ptr: u32,
    /// The input value.
    pub input: u32,
    /// The output value.
    pub output: u32,
    /// Memory records for reading the input.
    pub input_memory_record: MemoryReadRecord,
    /// Memory records for writing the output.
    pub output_memory_record: MemoryWriteRecord,
    /// Local memory accesses.
    pub local_mem_access: Vec<MemoryLocalEvent>,
}
