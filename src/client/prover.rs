use crate::crypto::poseidon::PoseidonHasher;
use crate::error::DenseZKError;
use crate::rel1cs::types::GraphEdge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientWitness {
    pub edge: GraphEdge,
    pub commitment: String,
}

pub struct DenseClient {
    hasher: PoseidonHasher,
}

impl DenseClient {
    pub fn new() -> Self {
        Self {
            hasher: PoseidonHasher::new(),
        }
    }

    pub fn create_witness(
        &mut self,
        sender: u64,
        receiver: u64,
        weight: u64,
    ) -> Result<ClientWitness, DenseZKError> {
        let commitment = self.hasher.hash_edge(sender, receiver, weight)?;

        let edge = GraphEdge {
            sender_id: sender,
            receiver_id: receiver,
            weight,
        };

        Ok(ClientWitness { edge, commitment })
    }
}

pub mod network;

pub use network::ProverNetworkConfig;
pub use network::ProverNetworkClientAsync;

#[cfg(feature = "sync")]
pub use network::ProverNetworkClientSync;
