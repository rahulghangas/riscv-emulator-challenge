use std::time::Instant;

use alloy_primitives::{hex::FromHex, B256};
use sp1_core_executor::{Executor, Program};

fn main() {
    // Load the program.
    let program = include_bytes!("../../artifacts/rsp");

    // Load the input.
    let buffer = include_bytes!("../../artifacts/buffer.bin");

    // Number of runs for benchmarking.
    const NUM_RUNS: usize = 5;
    let mut total_elapsed = 0.0;
    let mut total_mhz = 0.0;

    // Run the benchmark multiple times.
    for i in 0..NUM_RUNS {
        println!("Run {}/{}", i + 1, NUM_RUNS);
        
        // Setup the executor.
        let mut executor = Executor::new(Program::from(program).unwrap());
        executor.write_stdin_slice(buffer);

        // Run the executor.
        let start = Instant::now();
        executor.run().unwrap();
        let elapsed = start.elapsed().as_secs_f64();

        // Read the outputs.
        let mut first = [0u8; 8];
        executor.read_public_values_slice(&mut first);

        let mut bytes = [0u8; 32];
        executor.read_public_values_slice(&mut bytes);

        let block_hash = B256::from_slice(&bytes);
        assert_eq!(
            block_hash,
            B256::from_hex("dab3111c08b6a9330098afd5bb0f690b20871522a1f21c924a2aabc6dbd6a5b9").unwrap()
        );
        
        let mhz = (executor.state.global_clk as f64 / elapsed) / 1_000_000.0;
        
        println!("  block_hash={block_hash}");
        println!("  cycles: {}", executor.state.global_clk);
        println!("  elapsed: {:.4} seconds", elapsed);
        println!("  mhz: {:.2}", mhz);
        
        // Accumulate totals.
        total_elapsed += elapsed;
        total_mhz += mhz;
    }

    // Calculate and print averages
    let avg_elapsed = total_elapsed / NUM_RUNS as f64;
    let avg_mhz = total_mhz / NUM_RUNS as f64;

    println!("\n===== BENCHMARK RESULTS =====");
    println!("Runs: {}", NUM_RUNS);
    println!("Average elapsed: {:.4} seconds", avg_elapsed);
    println!("Average MHz: {:.2}", avg_mhz);
}
