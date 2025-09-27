/// Utility functions for path operations and file handling
pub struct PathUtils;

impl PathUtils {
    /// Extract object name from file path (e.g., "objects/player.moo" -> "player")
    /// 
    /// This function extracts the base name of a file and removes the .moo extension
    /// if present. It's commonly used to get object names from MOO file paths.
    /// 
    /// # Arguments
    /// * `path` - The file path to extract the object name from
    /// 
    /// # Returns
    /// * `Some(String)` - The object name without extension if the file has a .moo extension
    /// * `None` - If the path doesn't contain a valid .moo file
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// 
    /// assert_eq!(PathUtils::extract_object_name_from_path("objects/player.moo"), Some("player".to_string()));
    /// assert_eq!(PathUtils::extract_object_name_from_path("room.moo"), Some("room".to_string()));
    /// assert_eq!(PathUtils::extract_object_name_from_path("regular.txt"), None);
    /// assert_eq!(PathUtils::extract_object_name_from_path("objects/player"), None);
    /// ```
    pub fn extract_object_name_from_path(path: &str) -> Option<String> {
        if let Some(filename) = std::path::Path::new(path).file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.ends_with(".moo") {
                    return Some(filename_str.trim_end_matches(".moo").to_string());
                }
            }
        }
        None
    }
    
    /// Extract filename without extension from a file path
    /// 
    /// This is a more general version that works with any file extension,
    /// not just .moo files.
    /// 
    /// # Arguments
    /// * `path` - The file path to extract the filename from
    /// 
    /// # Returns
    /// * `Some(String)` - The filename without extension
    /// * `None` - If the path doesn't contain a valid filename
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// 
    /// assert_eq!(PathUtils::extract_filename_without_extension("objects/player.moo"), Some("player".to_string()));
    /// assert_eq!(PathUtils::extract_filename_without_extension("config.json"), Some("config".to_string()));
    /// assert_eq!(PathUtils::extract_filename_without_extension("README"), Some("README".to_string()));
    /// ```
    pub fn extract_filename_without_extension(path: &str) -> Option<String> {
        if let Some(filename) = std::path::Path::new(path).file_name() {
            if let Some(filename_str) = filename.to_str() {
                if let Some(stem) = std::path::Path::new(filename_str).file_stem() {
                    return stem.to_str().map(|s| s.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_object_name_from_path() {
        // Test basic .moo file extraction
        assert_eq!(PathUtils::extract_object_name_from_path("objects/player.moo"), Some("player".to_string()));
        assert_eq!(PathUtils::extract_object_name_from_path("room.moo"), Some("room".to_string()));
        
        // Test nested paths
        assert_eq!(PathUtils::extract_object_name_from_path("objects/rooms/lobby.moo"), Some("lobby".to_string()));
        
        // Test files without .moo extension
        assert_eq!(PathUtils::extract_object_name_from_path("regular.txt"), None);
        assert_eq!(PathUtils::extract_object_name_from_path("objects/player"), None);
        
        // Test edge cases
        assert_eq!(PathUtils::extract_object_name_from_path(""), None);
        assert_eq!(PathUtils::extract_object_name_from_path(".moo"), Some("".to_string()));
    }

    #[test]
    fn test_extract_filename_without_extension() {
        // Test .moo files
        assert_eq!(PathUtils::extract_filename_without_extension("objects/player.moo"), Some("player".to_string()));
        
        // Test other extensions
        assert_eq!(PathUtils::extract_filename_without_extension("config.json"), Some("config".to_string()));
        assert_eq!(PathUtils::extract_filename_without_extension("README.md"), Some("README".to_string()));
        
        // Test files without extension
        assert_eq!(PathUtils::extract_filename_without_extension("README"), Some("README".to_string()));
        assert_eq!(PathUtils::extract_filename_without_extension("Makefile"), Some("Makefile".to_string()));
        
        // Test edge cases
        assert_eq!(PathUtils::extract_filename_without_extension(""), None);
        assert_eq!(PathUtils::extract_filename_without_extension(".hidden"), Some(".hidden".to_string()));
    }
}
