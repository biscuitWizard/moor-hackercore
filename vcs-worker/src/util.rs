/// Utility functions for working with timestamps and time-related operations

/// Get the current Unix timestamp in seconds
pub fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_generation() {
        let timestamp = current_unix_timestamp();
        assert!(timestamp > 0);
        
        // Should be a reasonable Unix timestamp (after year 2020)
        assert!(timestamp > 1577836800); // 2020-01-01
    }
}
