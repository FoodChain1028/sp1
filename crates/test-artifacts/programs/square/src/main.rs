#![no_main]

// should create a new syscall in the executor
// use sp1_zkvm::syscalls::syscall_square;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let n = 10;
    // Compute the n'th fibonacci number, using normal Rust code.
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow.
        a = b;
        b = c;
    }
    sp1_zkvm::io::commit(&a);
    sp1_zkvm::io::commit(&b);
}
