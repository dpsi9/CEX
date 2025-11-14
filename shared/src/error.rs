pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum CexError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("redis error: {0}")]
    Redis(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}
