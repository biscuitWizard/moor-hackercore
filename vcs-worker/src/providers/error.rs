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
    #[error("Reference not found: {0}")]
    ReferenceNotFound(String),
    #[error("Change not found: {0}")]
    #[allow(dead_code)]
    ChangeNotFound(String),
    #[error("Version not found for object: {0}")]
    VersionNotFound(String),
    #[error("Invalid version: expected at least 1, got {0}")]
    #[allow(dead_code)]
    InvalidVersion(u64),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Alias for Result using ProviderError
pub type ProviderResult<T> = Result<T, ProviderError>;

