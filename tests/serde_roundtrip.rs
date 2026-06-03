use densezk_sdk::client::prover::ClientWitness;
use densezk_sdk::{DenseClient, LocalProver, PublicInputs, ZKProof};

#[test]
fn client_witness_json_roundtrip() {
    let mut client = DenseClient::new();
    let witness = client.create_witness(123, 456, 7).expect("create witness");
    let json = serde_json::to_string(&witness).expect("serialize witness");
    let roundtrip: ClientWitness = serde_json::from_str(&json).expect("deserialize witness");
    assert_eq!(witness, roundtrip);
}

#[test]
fn public_inputs_json_roundtrip() {
    let public_inputs = PublicInputs {
        graph_root: "0xdeadbeef".to_string(),
        threshold: 42,
    };
    let json = serde_json::to_string(&public_inputs).expect("serialize public inputs");
    let roundtrip: PublicInputs = serde_json::from_str(&json).expect("deserialize public inputs");
    assert_eq!(public_inputs, roundtrip);
}

#[test]
fn zkproof_json_roundtrip() {
    let prover = LocalProver::setup().expect("setup prover");
    let mut client = DenseClient::new();

    let witness = client.create_witness(10, 20, 30).expect("create witness");
    let public_inputs = PublicInputs {
        graph_root: "0xabc123".to_string(),
        threshold: 1,
    };

    let proof: ZKProof = prover
        .prove(witness, public_inputs)
        .expect("generate proof");

    let json = serde_json::to_string(&proof).expect("serialize proof");
    let roundtrip: ZKProof = serde_json::from_str(&json).expect("deserialize proof");
    assert_eq!(proof, roundtrip);
}
