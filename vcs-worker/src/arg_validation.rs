use moor_common::tasks::WorkerError;
use moor_var::Var;

/// Utility functions for argument validation
pub struct ArgValidation;

impl ArgValidation {
    /// Validate that we have at least the required number of arguments
    pub fn require_args(arguments: &[Var], required: usize, operation: &str) -> Result<(), WorkerError> {
        if arguments.len() < required {
            return Err(WorkerError::RequestError(
                format!("{} requires at least {} arguments", operation, required - 1)
            ));
        }
        Ok(())
    }
    
    /// Extract a string argument at the given index
    pub fn extract_string(arguments: &[Var], index: usize, arg_name: &str) -> Result<String, WorkerError> {
        arguments[index].as_string().ok_or_else(|| {
            WorkerError::RequestError(format!("Argument {} must be a string ({})", index + 1, arg_name))
        }).map(|s| s.to_string())
    }
    
    /// Extract a list of strings from arguments starting at the given index
    pub fn extract_string_list(arguments: &[Var], start_index: usize, arg_name: &str) -> Result<Vec<String>, WorkerError> {
        let mut strings = Vec::new();
        for i in start_index..arguments.len() {
            let string_val = arguments[i].as_string().ok_or_else(|| {
                WorkerError::RequestError(format!("Argument {} must be a string ({})", i + 1, arg_name))
            })?;
            strings.push(string_val.to_string());
        }
        Ok(strings)
    }
    
    /// Extract a boolean argument at the given index, with default value
    pub fn extract_bool_or_default(arguments: &[Var], index: usize, default: bool) -> bool {
        arguments.get(index)
            .and_then(|arg| arg.as_bool())
            .unwrap_or(default)
    }
    
    /// Extract an integer argument at the given index, with default value
    pub fn extract_int_or_default(arguments: &[Var], index: usize, default: Option<usize>) -> Option<usize> {
        arguments.get(index)
            .and_then(|arg| arg.as_integer().map(|i| i as usize))
            .or(default)
    }
}
