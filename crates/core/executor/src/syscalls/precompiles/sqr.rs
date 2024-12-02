use crate::{
    events::{LookupId, PrecompileEvent, SquareEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

pub(crate) struct SquareSyscall;

impl Syscall for SquareSyscall {
    fn execute(
        &self,
        rt: &mut SyscallContext,
        syscall_code: SyscallCode,
        arg1: u32,
        arg2: u32,
    ) -> Option<u32> {
        let clk = rt.clk;
        let input_ptr = arg1;
        let output_ptr = arg2;

        // Reading the input value.
        let (input_mem_record, input) = rt.mr(input_ptr);

        // Calculating the square.
        let output = input * input;

        // Writing the output value.
        rt.clk += 1; // the writing happens in the next cycle.
        let output_mem_record = rt.mw(output_ptr, output);

        // Create a square event.
        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard(); // for recursion (?)
        let event = PrecompileEvent::Square(SquareEvent {
            lookup_id,
            shard,
            clk,
            input_ptr,
            output_ptr,
            input,
            output,
            input_memory_record: input_mem_record,
            output_memory_record: output_mem_record,
            local_mem_access: rt.postprocess(),
        });

        let syscall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, syscall_event, event);

        None
    }

    fn num_extra_cycles(&self) -> u32 {
        1
    }
}
