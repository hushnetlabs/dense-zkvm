use ark_bn254::{Bn254, Fr};
use ark_groth16::{prepare_verifying_key, Groth16, ProvingKey, VerifyingKey};
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use ark_std::rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::client::prover::ClientWitness;
use crate::error::DenseZKError;
use crate::rel1cs::types::PublicInputs;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZKProof {
    pub proof_bytes: Vec<u8>,
    pub root_commitment: String,
}

struct DenseZKCircuit {
    witness: ClientWitness,
    public_inputs: PublicInputs,
}

impl ConstraintSynthesizer<Fr> for DenseZKCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let sender = cs.new_witness_variable(|| Ok(Fr::from(self.witness.edge.sender_id)))?;
        let receiver = cs.new_witness_variable(|| Ok(Fr::from(self.witness.edge.receiver_id)))?;
        let weight = cs.new_witness_variable(|| Ok(Fr::from(self.witness.edge.weight)))?;

        let threshold = Fr::from(self.public_inputs.threshold);
        let threshold_pub = cs.new_input_variable(|| Ok(threshold))?;

        let one = cs.new_witness_variable(|| Ok(Fr::from(1u64)))?;

        cs.enforce_constraint(lc!() + sender, lc!() + one, lc!() + sender)?;

        cs.enforce_constraint(lc!() + receiver, lc!() + one, lc!() + receiver)?;

        cs.enforce_constraint(lc!() + weight, lc!() + one, lc!() + weight)?;

        cs.enforce_constraint(lc!() + threshold_pub, lc!() + one, lc!() + threshold_pub)?;

        Ok(())
    }
}

pub struct LocalProver {
    proving_key: ProvingKey<Bn254>,
    verifying_key: VerifyingKey<Bn254>,
}

impl LocalProver {
    pub fn setup() -> Result<Self, DenseZKError> {
        let mut rng = thread_rng();

        let dummy_circuit = DenseZKCircuit {
            witness: ClientWitness {
                edge: crate::rel1cs::types::GraphEdge {
                    sender_id: 0,
                    receiver_id: 0,
                    weight: 0,
                },
                commitment: String::new(),
            },
            public_inputs: PublicInputs {
                graph_root: String::new(),
                threshold: 0,
            },
        };

        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(dummy_circuit, &mut rng)
            .map_err(|_| DenseZKError::SetupFailed)?;

        Ok(Self {
            proving_key: pk,
            verifying_key: vk,
        })
    }

    pub fn prove(
        &self,
        witness: ClientWitness,
        public_inputs: PublicInputs,
    ) -> Result<ZKProof, DenseZKError> {
        let mut rng = thread_rng();

        let circuit = DenseZKCircuit {
            witness,
            public_inputs: public_inputs.clone(),
        };

        let proof = Groth16::<Bn254>::prove(&self.proving_key, circuit, &mut rng)
            .map_err(|_| DenseZKError::WitnessGenerationFailed)?;

        let pvk = prepare_verifying_key(&self.verifying_key);
        let public_input_values: Vec<Fr> = vec![Fr::from(public_inputs.threshold)];

        let valid = Groth16::<Bn254>::verify_proof(&pvk, &proof, &public_input_values)
            .map_err(|_| DenseZKError::ConstraintViolation)?;

        if !valid {
            return Err(DenseZKError::ConstraintViolation);
        }

        let mut proof_bytes = Vec::new();
        proof
            .serialize_compressed(&mut proof_bytes)
            .map_err(|_| DenseZKError::WitnessGenerationFailed)?;

        Ok(ZKProof {
            proof_bytes,
            root_commitment: public_inputs.graph_root,
        })
    }
}
