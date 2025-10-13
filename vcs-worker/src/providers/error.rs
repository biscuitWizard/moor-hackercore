use thiserror::Error;

/// Error type for provider operations
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Fjall database error: {0}")]
    FjallError(#[from] fjall::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Compilation error: {0}")]
    CompilationError(#[from] moor_compiler::ObjDefParseError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Object not found: {0}")]
    #[allow(dead_code)]
    ObjectNotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Alias for Result using ProviderError
pub type ProviderResult<T> = Result<T, ProviderError>;
