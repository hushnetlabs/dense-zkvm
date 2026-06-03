use crate::client::prover::ClientWitness;
use crate::error::DenseZKError;
use crate::prover::local::ZKProof;
use crate::rel1cs::types::PublicInputs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverNetworkConfig {
    pub base_url: String,
    pub max_concurrency: u32,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub retry_backoff_ms: u64,
}

impl Default for ProverNetworkConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            max_concurrency: 4,
            timeout_secs: 30,
            retry_count: 3,
            retry_backoff_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveRequest {
    pub sender_id: u64,
    pub receiver_id: u64,
    pub weight: u64,
    pub graph_root: String,
    pub threshold: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub proof_bytes: Vec<u8>,
    pub root_commitment: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[cfg(feature = "async")]
pub mod async_client {
    use super::*;
    use reqwest::Client;
    use tokio::sync::Semaphore;
    use tokio::time::{sleep, Duration};

    pub struct ProverNetworkClientAsync {
        config: ProverNetworkConfig,
        client: Client,
        semaphore: Semaphore,
    }

    impl ProverNetworkClientAsync {
        pub fn new(config: ProverNetworkConfig) -> Result<Self, DenseZKError> {
            let client = Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            let semaphore = Semaphore::new(config.max_concurrency as usize);

            Ok(Self {
                config,
                client,
                semaphore,
            })
        }

        pub fn with_defaults() -> Result<Self, DenseZKError> {
            Self::new(ProverNetworkConfig::default())
        }

        async fn send_prove_request(&self, request: &ProveRequest) -> Result<ZKProof, DenseZKError> {
            let url = format!("{}/prove", self.config.base_url);

            let response = self
                .client
                .post(&url)
                .json(request)
                .send()
                .await
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            if !response.status().is_success() {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .unwrap_or(ErrorResponse {
                        error: "Unknown error".to_string(),
                    });
                return Err(DenseZKError::NetworkError(error.error));
            }

            let prove_response: ProveResponse = response
                .json()
                .await
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            Ok(ZKProof {
                proof_bytes: prove_response.proof_bytes,
                root_commitment: prove_response.root_commitment,
            })
        }

        pub async fn prove(&self, witness: ClientWitness, public_inputs: PublicInputs) -> Result<ZKProof, DenseZKError> {
            let request = ProveRequest {
                sender_id: witness.edge.sender_id,
                receiver_id: witness.edge.receiver_id,
                weight: witness.edge.weight,
                graph_root: public_inputs.graph_root,
                threshold: public_inputs.threshold,
            };

            let _permit = self
                .semaphore
                .acquire()
                .await
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            let mut last_error = None;
            for attempt in 0..self.config.retry_count {
                match self.send_prove_request(&request).await {
                    Ok(proof) => return Ok(proof),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < self.config.retry_count - 1 {
                            sleep(Duration::from_millis(self.config.retry_backoff_ms * (attempt + 1) as u64)).await;
                        }
                    }
                }
            }

            Err(last_error.unwrap_or(DenseZKError::WitnessGenerationFailed))
        }

        pub async fn prove_batch(
            &self,
            witnesses: Vec<ClientWitness>,
            public_inputs: Vec<PublicInputs>,
        ) -> Result<Vec<ZKProof>, DenseZKError> {
            if witnesses.len() != public_inputs.len() {
                return Err(DenseZKError::NetworkError(
                    "witnesses and public_inputs must have equal length".to_string(),
                ));
            }

            let count = witnesses.len();
            let mut handles = Vec::with_capacity(count);

            for (witness, inputs) in witnesses.into_iter().zip(public_inputs.into_iter()) {
                let this = Self {
                    config: self.config.clone(),
                    client: self.client.clone(),
                    semaphore: Semaphore::new(self.config.max_concurrency as usize),
                };
                handles.push(tokio::spawn(async move { this.prove(witness, inputs).await }));
            }

            let mut proofs = Vec::with_capacity(count);
            let mut failures = Vec::new();

            for handle in handles {
                match handle.await {
                    Ok(Ok(proof)) => proofs.push(proof),
                    Ok(Err(e)) => failures.push(e.to_string()),
                    Err(_) => failures.push("Task panicked".to_string()),
                }
            }

            if failures.is_empty() {
                Ok(proofs)
            } else if proofs.is_empty() {
                Err(DenseZKError::PartialFailure(
                    format!("{} failures: {}", failures.len(), failures.join("; ")),
                    count,
                ))
            } else {
                Err(DenseZKError::PartialFailure(
                    format!(
                        "{} failures: {}. {} succeeded",
                        failures.len(),
                        failures.join("; "),
                        proofs.len()
                    ),
                    count,
                ))
            }
        }

        pub fn config(&self) -> &ProverNetworkConfig {
            &self.config
        }
    }

    impl Clone for ProverNetworkClientAsync {
        fn clone(&self) -> Self {
            Self {
                config: self.config.clone(),
                client: self.client.clone(),
                semaphore: Semaphore::new(self.config.max_concurrency as usize),
            }
        }
    }
}

#[cfg(feature = "async")]
pub use async_client::ProverNetworkClientAsync;

#[cfg(feature = "sync")]
pub mod sync_client {
    use super::*;
    use crate::prover::local::ZKProof;
    use std::sync::mpsc;
    use std::thread;

    pub struct ProverNetworkClientSync {
        config: ProverNetworkConfig,
    }

    impl ProverNetworkClientSync {
        pub fn new(config: ProverNetworkConfig) -> Result<Self, DenseZKError> {
            Ok(Self { config })
        }

        pub fn with_defaults() -> Result<Self, DenseZKError> {
            Self::new(ProverNetworkConfig::default())
        }

        fn send_prove_request(&self, request: &ProveRequest) -> Result<ZKProof, DenseZKError> {
            let url = format!("{}/prove", self.config.base_url);

            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
                .build()
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            let response = client
                .post(&url)
                .json(request)
                .send()
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            if !response.status().is_success() {
                let error: ErrorResponse = response
                    .json()
                    .unwrap_or(ErrorResponse {
                        error: "Unknown error".to_string(),
                    });
                return Err(DenseZKError::NetworkError(error.error));
            }

            let prove_response: ProveResponse = response
                .json()
                .map_err(|e| DenseZKError::NetworkError(e.to_string()))?;

            Ok(ZKProof {
                proof_bytes: prove_response.proof_bytes,
                root_commitment: prove_response.root_commitment,
            })
        }

        pub fn prove(&self, witness: ClientWitness, public_inputs: PublicInputs) -> Result<ZKProof, DenseZKError> {
            let request = ProveRequest {
                sender_id: witness.edge.sender_id,
                receiver_id: witness.edge.receiver_id,
                weight: witness.edge.weight,
                graph_root: public_inputs.graph_root,
                threshold: public_inputs.threshold,
            };

            let mut last_error = None;
            for attempt in 0..self.config.retry_count {
                match self.send_prove_request(&request) {
                    Ok(proof) => return Ok(proof),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < self.config.retry_count - 1 {
                            std::thread::sleep(std::time::Duration::from_millis(
                                self.config.retry_backoff_ms * (attempt + 1) as u64,
                            ));
                        }
                    }
                }
            }

            Err(last_error.unwrap_or(DenseZKError::WitnessGenerationFailed))
        }

        pub fn prove_batch(
            &self,
            witnesses: Vec<ClientWitness>,
            public_inputs: Vec<PublicInputs>,
        ) -> Result<Vec<ZKProof>, DenseZKError> {
            if witnesses.len() != public_inputs.len() {
                return Err(DenseZKError::NetworkError(
                    "witnesses and public_inputs must have equal length".to_string(),
                ));
            }

            let count = witnesses.len();
            let (tx, rx) = mpsc::channel();
            let mut handles = Vec::new();

            for (witness, inputs) in witnesses.into_iter().zip(public_inputs.into_iter()) {
                let config = self.config.clone();
                let tx = tx.clone();
                let handle = thread::spawn(move || {
                    let client = Self { config };
                    let result = client.prove(witness, inputs);
                    let _ = tx.send(result);
                });
                handles.push(handle);
            }

            drop(tx);

            let mut proofs = Vec::new();
            let mut failures = Vec::new();

            for result in rx.iter().take(count) {
                match result {
                    Ok(proof) => proofs.push(proof),
                    Err(e) => failures.push(e.to_string()),
                }
            }

            for handle in handles {
                let _ = handle.join();
            }

            if failures.is_empty() {
                Ok(proofs)
            } else if proofs.is_empty() {
                Err(DenseZKError::PartialFailure(
                    format!("{} failures: {}", failures.len(), failures.join("; ")),
                    count,
                ))
            } else {
                Err(DenseZKError::PartialFailure(
                    format!(
                        "{} failures: {}. {} succeeded",
                        failures.len(),
                        failures.join("; "),
                        proofs.len()
                    ),
                    count,
                ))
            }
        }

        pub fn config(&self) -> &ProverNetworkConfig {
            &self.config
        }
    }

    impl Clone for ProverNetworkClientSync {
        fn clone(&self) -> Self {
            Self {
                config: self.config.clone(),
            }
        }
    }
}

#[cfg(feature = "sync")]
pub use sync_client::ProverNetworkClientSync;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_network_config_defaults() {
        let config = ProverNetworkConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.max_concurrency, 4);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_backoff_ms, 100);
    }

    #[test]
    fn test_prover_network_config_custom() {
        let config = ProverNetworkConfig {
            base_url: "http://192.168.1.100:9090".to_string(),
            max_concurrency: 8,
            timeout_secs: 60,
            retry_count: 5,
            retry_backoff_ms: 200,
        };
        assert_eq!(config.base_url, "http://192.168.1.100:9090");
        assert_eq!(config.max_concurrency, 8);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.retry_count, 5);
        assert_eq!(config.retry_backoff_ms, 200);
    }

    #[test]
    fn test_prove_request_serialization() {
        let request = ProveRequest {
            sender_id: 456,
            receiver_id: 789,
            weight: 1,
            graph_root: "0xabc123".to_string(),
            threshold: 1,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("456"));
        assert!(json.contains("789"));
        assert!(json.contains("0xabc123"));
    }

    #[test]
    fn test_prove_request_clone() {
        let request = ProveRequest {
            sender_id: 456,
            receiver_id: 789,
            weight: 1,
            graph_root: "0xabc123".to_string(),
            threshold: 1,
        };
        let cloned = request.clone();
        assert_eq!(request.sender_id, cloned.sender_id);
        assert_eq!(request.receiver_id, cloned.receiver_id);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_prover_client_creation() {
        let client = ProverNetworkClientAsync::new(ProverNetworkConfig::default());
        assert!(client.is_ok());
    }

    #[cfg(feature = "sync")]
    #[test]
    fn test_sync_prover_client_creation() {
        let client = ProverNetworkClientSync::new(ProverNetworkConfig::default());
        assert!(client.is_ok());
    }
}