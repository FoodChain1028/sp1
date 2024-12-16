use crate::syscall::precompiles::keccak256::{STATE_NUM_WORDS, STATE_SIZE};
use crate::utils::{next_power_of_two, zeroed_f_vec};
use crate::{air::MemoryAirBuilder, memory::MemoryCols};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_field::PrimeField32;
use p3_keccak_air::generate_trace_rows;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use sp1_core_executor::events::PrecompileEvent;
use sp1_core_executor::syscalls::SyscallCode;
use sp1_core_executor::ExecutionRecord;
use sp1_core_executor::Program;
use sp1_stark::air::{InteractionScope, MachineAir, SP1AirBuilder};
use std::borrow::{Borrow, BorrowMut};

use super::{SquareChip, SquareCols, SQUARE_NUM_COLS};

impl<F> BaseAir<F> for SquareChip {
    fn width(&self) -> usize {
        SQUARE_NUM_COLS
    }
}

impl<AB> Air<AB> for SquareChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let (local, next) = (main.row_slice(0), main.row_slice(1));
        let local: &SquareCols<AB::Var> = (*local).borrow();
        let next: &SquareCols<AB::Var> = (*next).borrow();

        // Constrain the incrementing nonce
        builder.when_first_row().assert_zero(local.nonce);
        builder.when_transition().assert_eq(local.nonce + AB::Expr::one(), next.nonce);

        // Assert that is_real is boolean
        builder.assert_bool(local.is_real);

        // Verify square computation
        builder.when(local.is_real).assert_eq(local.output, local.input * local.input);

        // input memory access
        builder.eval_memory_access(
            local.shard,
            local.clk,
            local.input_ptr,
            &local.input_access,
            local.is_real,
        );

        // output memory access
        builder.eval_memory_access(
            local.shard,
            local.clk + AB::Expr::one(),
            local.output_ptr,
            &local.output_access,
            local.is_real,
        );

        // Receive syscall event
        builder.receive_syscall(
            local.shard,
            local.clk,
            local.nonce,
            AB::F::from_canonical_u32(SyscallCode::SQR.syscall_id()),
            local.input_ptr,
            local.output_ptr,
            local.is_real,
            InteractionScope::Local,
        );

        // Ensure values stay consistent in transition rows
        let mut transition_builder = builder.when_transition();
        transition_builder.assert_eq(local.shard, next.shard);
        transition_builder.assert_eq(local.clk + AB::Expr::one(), next.clk);
        transition_builder.assert_eq(local.is_real, next.is_real);
    }
}

impl<F> MachineAir<F> for SquareChip
where
    F: PrimeField32,
{
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "Square".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // took reference from keccak256::trace.rs
        // Init variables
        let events = input.get_precompile_events(SyscallCode::SQR);
        let num_events = events.len();
        let num_rows = if num_events > 0 { num_events.next_power_of_two() } else { 1 };

        // Init values
        let values = vec![0u32; num_rows * SQUARE_NUM_COLS];
        let mut values = unsafe { std::mem::transmute::<Vec<u32>, Vec<F>>(values) };

        // dummy rows
        let dummy_row = [F::zero(); SQUARE_NUM_COLS];

        // populate the trace matrix with actual Square events
        for (i, (_, event)) in events.iter().enumerate() {
            self.populate_chunk(
                event,
                &mut values[i * SQUARE_NUM_COLS..(i + 1) * SQUARE_NUM_COLS],
                &mut Vec::new(),
            );
        }

        // fill any remaining rows with dummy data to meet the power-of-two requirement
        for i in num_events..num_rows {
            values[i * SQUARE_NUM_COLS..(i + 1) * SQUARE_NUM_COLS].copy_from_slice(&dummy_row);
        }

        // Step 7: Convert the flat values vector into a RowMajorMatrix
        let mut trace = RowMajorMatrix::new(values, SQUARE_NUM_COLS);

        // Step 8: Assign a unique nonce to each row to ensure trace integrity
        for i in 0..trace.height() {
            let cols: &mut SquareCols<F> =
                trace.values[i * SQUARE_NUM_COLS..(i + 1) * SQUARE_NUM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.get_precompile_events(SyscallCode::SQR).is_empty()
    }
}
