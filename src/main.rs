use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use densezk_sdk::{execute_dense_zk_flow, PublicInputs, ZKProof};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Generate a DenseZK proof locally")]
struct Args {
    /// Save proof and public inputs to a JSON file
    #[arg(long, short)]
    output: Option<PathBuf>,
}

#[derive(Serialize)]
struct ProofOutputFile {
    proof: ZKProof,
    public_inputs: PublicInputs,
}

fn save_proof_output(path: &PathBuf, proof: ZKProof, public_inputs: PublicInputs) -> ExitCode {
    let output = ProofOutputFile {
        proof,
        public_inputs,
    };

    let json = match serde_json::to_string_pretty(&output) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to serialize proof output: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to write output file '{}': {}", path.display(), e);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = file.write_all(json.as_bytes()) {
        eprintln!("Failed to write output file '{}': {}", path.display(), e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args = Args::parse();

    let sender = 456;
    let receiver = 789;
    let weight = 1;
    let graph_root = "0xabc123";
    let threshold = 1;

    let result = execute_dense_zk_flow(sender, receiver, weight, graph_root, threshold);

    match result {
        Ok(proof) => {
            println!(
                "Proof generated locally, size: {} bytes",
                proof.proof_bytes.len()
            );

            if let Some(path) = args.output {
                return save_proof_output(&path, proof, PublicInputs {
                    graph_root: graph_root.to_string(),
                    threshold,
                });
            }

            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error during execution: {}", e);
            ExitCode::FAILURE
        }
    }
}
