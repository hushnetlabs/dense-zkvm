use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub sender_id: u64,
    pub receiver_id: u64,
    pub weight: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicInputs {
    pub graph_root: String,
    pub threshold: u64,
}
