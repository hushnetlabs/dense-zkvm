use densezk_sdk::{
    DenseClient, DenseZKError, ProverNetworkClientSync, ProverNetworkConfig, PublicInputs, ZKProof,
};
use std::env;
use std::time::Instant;

fn main() {
    println!("DenseZK Network Benchmark - Parallel Prover Client (Sync)\n");
    println!("=============================================================\n");

    let base_url = env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("Target server: {}\n", base_url);

    let concurrency_levels = vec![1, 2, 4, 8];

    println!("Configuration:");
    println!("  - Target: {}", base_url);
    println!("  - Concurrency levels: {:?}", concurrency_levels);
    println!("  - Requests per level: 8\n");

    println!("Running parallel benchmark...\n");

    for concurrency in concurrency_levels {
        let num_requests = 8;
        let config = ProverNetworkConfig {
            base_url: base_url.clone(),
            max_concurrency: concurrency,
            timeout_secs: 30,
            retry_count: 2,
            retry_backoff_ms: 50,
        };

        let client = match ProverNetworkClientSync::new(config) {
            Ok(c) => c,
            Err(e) => {
                println!("[Error] Failed to create client: {}", e);
                continue;
            }
        };

        let mut witnesses = Vec::new();
        let mut public_inputs_list = Vec::new();

        for i in 0..num_requests {
            let mut c = DenseClient::new();
            let witness = c.create_witness(456 + i as u64, 789, 1).unwrap();
            let public_inputs = PublicInputs {
                graph_root: "0xabc123".to_string(),
                threshold: 1,
            };
            witnesses.push(witness);
            public_inputs_list.push(public_inputs);
        }

        let start = Instant::now();

        let results = client.prove_batch(witnesses, public_inputs_list);

        let elapsed = start.elapsed();

        match results {
            Ok(proofs) => {
                println!(
                    "  [{:2} concurrent] {} requests in {:4}ms (avg {:4}ms/req) - {} success",
                    concurrency,
                    num_requests,
                    elapsed.as_millis(),
                    elapsed.as_millis() / num_requests as u128,
                    proofs.len()
                );
            }
            Err(e) => {
                println!(
                    "  [{:2} concurrent] {} requests in {:4}ms - failed: {}",
                    concurrency,
                    num_requests,
                    elapsed.as_millis(),
                    e
                );
            }
        }
    }

    println!("\n--- Expected Results (with actual prover network) ---");
    println!("  1 connection:  baseline (~800ms for 8 requests @ 100ms each)");
    println!("  2 connections: ~1.8x speedup (400ms)");
    println!("  4 connections: ~3.5x speedup (230ms)");
    println!("  8 connections: ~4x speedup (200ms)");
    println!("\nNote: Speedup is limited by server latency per request.");
}
