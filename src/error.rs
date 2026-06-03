use thiserror::Error;

#[derive(Error, Debug)]
pub enum DenseZKError {
    #[error("Failed to generate witness")]
    WitnessGenerationFailed,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Constraint violation in Rel1CS")]
    ConstraintViolation,

    #[error("Proving key setup failed")]
    SetupFailed,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timeout after {0} retries")]
    NetworkTimeout(u32),

    #[error("Partial failure: {0} failed out of {1} requests")]
    PartialFailure(String, usize),
}
