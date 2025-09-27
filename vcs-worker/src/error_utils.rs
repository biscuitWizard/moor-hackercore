use moor_common::tasks::WorkerError;
use moor_var::v_str;

/// Utility functions for common error handling patterns
pub struct ErrorUtils;

impl ErrorUtils {
    /// Create a standardized "Git repository not available" error
    pub fn git_repo_not_available(path: Option<&str>) -> WorkerError {
        let repo_path = path.unwrap_or("/game");
        WorkerError::RequestError(format!("Git repository not available at {}", repo_path))
    }
    
    /// Create a standardized "Git repository not available" error for SSH operations
    pub fn git_repo_not_available_ssh() -> WorkerError {
        WorkerError::RequestError("Git repository not available".to_string())
    }
    
    /// Create a standardized error message for failed operations
    pub fn operation_failed(operation: &str, error: &str) -> WorkerError {
        WorkerError::RequestError(format!("Failed to {}: {}", operation, error))
    }
    
    /// Create a standardized success message
    pub fn success_message(message: &str) -> Vec<moor_var::Var> {
        vec![v_str(message)]
    }
    
    /// Create a standardized success message with formatting
    pub fn success_message_formatted(_template: &str, args: &[&str]) -> Vec<moor_var::Var> {
        vec![v_str(&format!("{}", args.join(", ")))]
    }
}
