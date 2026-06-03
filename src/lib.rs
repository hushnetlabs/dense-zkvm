pub mod client;
pub mod crypto;
pub mod error;
pub mod prover;
pub mod rel1cs;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use client::prover::DenseClient;
pub use client::prover::ProverNetworkConfig;
pub use client::prover::ProverNetworkClientAsync;
#[cfg(feature = "sync")]
pub use client::prover::ProverNetworkClientSync;
pub use error::DenseZKError;
pub use prover::local::{LocalProver, ZKProof};
pub use rel1cs::types::{GraphEdge, PublicInputs};

pub fn execute_dense_zk_flow(
    sender: u64,
    receiver: u64,
    weight: u64,
    graph_root: &str,
    threshold: u64,
) -> Result<ZKProof, DenseZKError> {
    let mut client = DenseClient::new();
    let witness = client.create_witness(sender, receiver, weight)?;

    println!(
        "[Client] Witness generated with Poseidon commitment: {}",
        witness.commitment
    );

    let public_inputs = PublicInputs {
        graph_root: graph_root.to_string(),
        threshold,
    };

    let prover = LocalProver::setup()?;
    let proof = prover.prove(witness, public_inputs)?;

    println!(
        "[Prover] Proof generated locally, size: {} bytes",
        proof.proof_bytes.len()
    );

    Ok(proof)
}
