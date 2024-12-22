mod air;

pub use air::*;
use p3_field::PrimeField32;
use sp1_core_executor::events::{ByteLookupEvent, PrecompileEvent};
use sp1_derive::AlignedBorrow;

use crate::memory::MemoryReadWriteCols;

pub const SQUARE_NUM_COLS: usize = size_of::<SquareCols<u8>>();

#[derive(Default)]
pub struct SquareChip;

impl SquareChip {
    pub const fn new() -> Self {
        Self
    }

    pub fn populate_chunk<F: PrimeField32>(
        &self,
        event: &PrecompileEvent,
        chunk: &mut [F],
        new_byte_lookup_events: &mut Vec<ByteLookupEvent>,
    ) {
        if let PrecompileEvent::Square(event) = event {
            chunk[0] = F::from_canonical_u32(event.shard);
            chunk[1] = F::from_canonical_u32(event.clk);
            chunk[2] = F::from_canonical_u32(event.input_ptr);
            chunk[3] = F::from_canonical_u32(event.output_ptr);
            chunk[4] = F::from_canonical_u32(event.input);
            chunk[5] = F::from_canonical_u32(event.input * event.input);
            chunk[6] = F::one();
        } else {
            unreachable!("Expected a Square event");
        }
    }
}

// Define the columns for square.
#[derive(AlignedBorrow)]
#[repr(C)]
pub struct SquareCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The nonce of the operation. (don't know what this is for)
    pub nonce: T,

    /// The pointer to the input.
    pub input_ptr: T,

    /// The pointer to the output.
    pub output_ptr: T,

    /// The input value.
    pub input: T,

    /// The output value.
    pub output: T,

    /// Memory columns for reading input
    pub input_access: MemoryReadWriteCols<T>,

    /// Memory columns for writing output
    pub output_access: MemoryReadWriteCols<T>,

    pub is_real: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use sp1_core_executor::{
        syscalls::SyscallCode, ExecutionRecord, Executor, Instruction, Opcode, Program,
    };
    use sp1_stark::SP1CoreOpts;

    fn square_program() -> Program {
        let input_value = 5;
        let result_ptr = 100;

        let mut instructions = vec![
            // Init X29 with 5
            Instruction::new(Opcode::ADD, 29, 0, input_value, false, true),
        ];
        // Square 5 for 10 times
        for i in 0..10 {
            instructions.extend(vec![
                // Store current value in memory
                Instruction::new(Opcode::ADD, 30, 0, result_ptr + i * 4, false, true), // Calculate memory address
                Instruction::new(Opcode::SW, 29, 30, 0, false, true), // Store value in memory
                Instruction::new(Opcode::ADD, 5, 0, SyscallCode::SQR as u32, false, true),
                Instruction::new(Opcode::ADD, 10, 0, result_ptr + i * 4, false, true), // input ptr
                Instruction::new(Opcode::ADD, 11, 0, result_ptr + (i + 1) * 4, false, true), // output ptr
                // Square ECALL
                Instruction::new(Opcode::ECALL, 5, 10, 11, false, false),
                // Load the result back into x29 for next iteration
                Instruction::new(Opcode::LW, 29, 11, 0, false, true),
            ]);
        }

        Program::new(instructions, 0, 0)
    }

    #[test]
    fn test_square_program_execute() {
        utils::setup_logger();
        let program = square_program();
        let mut runtime = Executor::new(program, SP1CoreOpts::default());
        runtime.run().unwrap();
    }

    // #[test]
    // fn test_square_program_prove() {
    //     utils::setup_logger();
    //     let program = square_program();
    //     run_test::<CpuProver<_, _>>(program).unwrap();
    // }
}
