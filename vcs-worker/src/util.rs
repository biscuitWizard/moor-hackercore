/// Utility functions for working with timestamps, time-related operations, and hashing
/// Get the current Unix timestamp in seconds
pub fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a Blake3 hash ID for a change based on its content
/// This ensures changes with the same content get the same ID (content-addressable)
pub fn generate_change_id(
    name: &str,
    description: Option<&str>,
    author: &str,
    timestamp: u64,
) -> String {
    let mut hasher = blake3::Hasher::new();
    
    // Hash the change components in a deterministic order
    hasher.update(name.as_bytes());
    hasher.update(b"\0"); // Separator
    hasher.update(description.unwrap_or("").as_bytes());
    hasher.update(b"\0"); // Separator
    hasher.update(author.as_bytes());
    hasher.update(b"\0"); // Separator
    hasher.update(&timestamp.to_le_bytes());
    
    // Return as hex string
    hasher.finalize().to_hex().to_string()
}

/// Get short form of a hash ID (first 12 characters for better collision resistance)
pub fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

/// Resolve a possibly-short hash to a full hash by searching the index
/// Returns Ok(Some(full_hash)) if found, Ok(None) if not found, Err if error
#[allow(dead_code)]
pub fn resolve_hash(
    short_or_full: &str,
    get_change_order: impl Fn() -> Result<Vec<String>, Box<dyn std::error::Error>>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // If it's already a full hash (64 chars for Blake3), return it
    if short_or_full.len() == 64 {
        return Ok(Some(short_or_full.to_string()));
    }
    
    // Otherwise, search for a hash that starts with this prefix
    let change_order = get_change_order()?;
    
    let mut matches: Vec<String> = change_order
        .into_iter()
        .filter(|hash| hash.starts_with(short_or_full))
        .collect();
    
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.remove(0))),
        _ => Err(format!(
            "Ambiguous hash prefix '{short_or_full}' matches multiple changes. Please provide more characters."
        )
        .into()),
    }
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

    #[test]
    fn test_generate_change_id() {
        let id1 = generate_change_id("test-change", Some("desc"), "author", 1234567890);
        let id2 = generate_change_id("test-change", Some("desc"), "author", 1234567890);
        let id3 = generate_change_id("different-change", Some("desc"), "author", 1234567890);
        
        // Same inputs should produce same hash
        assert_eq!(id1, id2);
        
        // Different inputs should produce different hash
        assert_ne!(id1, id3);
        
        // Should be Blake3 hash length (64 hex chars)
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn test_short_hash() {
        let full_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let short = short_hash(full_hash);
        
        assert_eq!(short, "abcdef123456");
        assert_eq!(short.len(), 12);
    }

    #[test]
    fn test_resolve_hash_full() {
        let full_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let changes = vec![full_hash.to_string()];
        
        let result = resolve_hash(full_hash, || Ok(changes.clone())).unwrap();
        assert_eq!(result, Some(full_hash.to_string()));
    }

    #[test]
    fn test_resolve_hash_short() {
        let full_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let changes = vec![full_hash.to_string()];
        
        let result = resolve_hash("abcdef", || Ok(changes.clone())).unwrap();
        assert_eq!(result, Some(full_hash.to_string()));
    }

    #[test]
    fn test_resolve_hash_ambiguous() {
        let hash1 = "abcdef1234567890000000000000000000000000000000000000000000000000";
        let hash2 = "abcdef1234567890111111111111111111111111111111111111111111111111";
        let changes = vec![hash1.to_string(), hash2.to_string()];
        
        let result = resolve_hash("abcdef123456", || Ok(changes.clone()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ambiguous"));
    }

    #[test]
    fn test_resolve_hash_not_found() {
        let changes = vec!["abcdef1234567890000000000000000000000000000000000000000000000000".to_string()];
        
        let result = resolve_hash("ffffff", || Ok(changes.clone())).unwrap();
        assert_eq!(result, None);
    }
}
