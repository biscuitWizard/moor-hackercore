use std::path::{Path, PathBuf};
use crate::config::Config;

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
    
    /// Get the path for a .meta file corresponding to a .moo file
    /// 
    /// This function converts a MOO file path to its corresponding meta file path
    /// by replacing the .moo extension with .meta.
    /// 
    /// # Arguments
    /// * `moo_path` - The path to the .moo file
    /// 
    /// # Returns
    /// * `PathBuf` - The corresponding .meta file path
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// 
    /// assert_eq!(PathUtils::meta_path("objects/player.moo"), PathBuf::from("objects/player.meta"));
    /// assert_eq!(PathUtils::meta_path("room.moo"), PathBuf::from("room.meta"));
    /// assert_eq!(PathUtils::meta_path("config"), PathBuf::from("config.meta"));
    /// ```
    pub fn meta_path<P: AsRef<Path>>(moo_path: P) -> PathBuf {
        let mut meta_path = moo_path.as_ref().to_path_buf();
        
        // Replace .moo extension with .meta
        if let Some(ext) = meta_path.extension() {
            if ext == "moo" {
                meta_path.set_extension("meta");
            } else {
                meta_path.set_extension("meta");
            }
        } else {
            meta_path.set_extension("meta");
        }
        
        meta_path
    }
    
    /// Get the full path for an object file in the objects directory
    /// 
    /// This function constructs the full path for a MOO object file, ensuring
    /// it has the .moo extension.
    /// 
    /// # Arguments
    /// * `work_dir` - The working directory of the repository
    /// * `config` - The configuration containing the objects directory
    /// * `object_name` - The name of the object (with or without .moo extension)
    /// 
    /// # Returns
    /// * `PathBuf` - The full path to the object file
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// use vcs_worker::Config;
    /// use std::path::PathBuf;
    /// 
    /// let work_dir = PathBuf::from("/game");
    /// let config = Config::default();
    /// assert_eq!(PathUtils::object_path(&work_dir, &config, "player"), 
    ///            PathBuf::from("/game/objects/player.moo"));
    /// assert_eq!(PathUtils::object_path(&work_dir, &config, "room.moo"), 
    ///            PathBuf::from("/game/objects/room.moo"));
    /// ```
    pub fn object_path<P: AsRef<Path>>(work_dir: P, config: &Config, object_name: &str) -> PathBuf {
        let mut path = work_dir.as_ref().join(config.objects_directory()).join(object_name);
        
        // Ensure the file has .moo extension
        if !path.extension().map_or(false, |ext| ext == "moo") {
            path.set_extension("moo");
        }
        
        path
    }
    
    /// Get the full path for a meta file in the objects directory
    /// 
    /// This function constructs the full path for a meta file corresponding
    /// to a MOO object file.
    /// 
    /// # Arguments
    /// * `work_dir` - The working directory of the repository
    /// * `config` - The configuration containing the objects directory
    /// * `object_name` - The name of the object (with or without .moo extension)
    /// 
    /// # Returns
    /// * `PathBuf` - The full path to the meta file
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// use vcs_worker::Config;
    /// use std::path::PathBuf;
    /// 
    /// let work_dir = PathBuf::from("/game");
    /// let config = Config::default();
    /// assert_eq!(PathUtils::object_meta_path(&work_dir, &config, "player"), 
    ///            PathBuf::from("/game/objects/player.meta"));
    /// assert_eq!(PathUtils::object_meta_path(&work_dir, &config, "room.moo"), 
    ///            PathBuf::from("/game/objects/room.meta"));
    /// ```
    pub fn object_meta_path<P: AsRef<Path>>(work_dir: P, config: &Config, object_name: &str) -> PathBuf {
        let mut path = work_dir.as_ref().join(config.objects_directory()).join(object_name);
        
        // Remove .moo extension if present and add .meta
        if path.extension().map_or(false, |ext| ext == "moo") {
            path.set_extension("meta");
        } else {
            path.set_extension("meta");
        }
        
        path
    }
    
    /// Ensure a filename has the .moo extension
    /// 
    /// This function ensures that a filename has the .moo extension,
    /// adding it if it's missing.
    /// 
    /// # Arguments
    /// * `filename` - The filename to ensure has .moo extension
    /// 
    /// # Returns
    /// * `String` - The filename with .moo extension
    /// 
    /// # Examples
    /// ```
    /// use vcs_worker::utils::PathUtils;
    /// 
    /// assert_eq!(PathUtils::ensure_moo_extension("player"), "player.moo");
    /// assert_eq!(PathUtils::ensure_moo_extension("room.moo"), "room.moo");
    /// ```
    pub fn ensure_moo_extension(filename: &str) -> String {
        if filename.ends_with(".moo") {
            filename.to_string()
        } else {
            format!("{}.moo", filename)
        }
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

    #[test]
    fn test_meta_path() {
        assert_eq!(PathUtils::meta_path("objects/player.moo"), PathBuf::from("objects/player.meta"));
        assert_eq!(PathUtils::meta_path("room.moo"), PathBuf::from("room.meta"));
        assert_eq!(PathUtils::meta_path("config"), PathBuf::from("config.meta"));
        assert_eq!(PathUtils::meta_path("objects/player"), PathBuf::from("objects/player.meta"));
    }

    #[test]
    fn test_object_path() {
        let work_dir = PathBuf::from("/game");
        let config = Config::default();
        
        assert_eq!(PathUtils::object_path(&work_dir, &config, "player"), 
                   PathBuf::from("/game/objects/player.moo"));
        assert_eq!(PathUtils::object_path(&work_dir, &config, "room.moo"), 
                   PathBuf::from("/game/objects/room.moo"));
        
        // Test with custom objects directory
        let mut custom_config = Config::default();
        custom_config.objects_directory = "custom_objects".to_string();
        assert_eq!(PathUtils::object_path(&work_dir, &custom_config, "test"), 
                   PathBuf::from("/game/custom_objects/test.moo"));
    }

    #[test]
    fn test_object_meta_path() {
        let work_dir = PathBuf::from("/game");
        let config = Config::default();
        
        assert_eq!(PathUtils::object_meta_path(&work_dir, &config, "player"), 
                   PathBuf::from("/game/objects/player.meta"));
        assert_eq!(PathUtils::object_meta_path(&work_dir, &config, "room.moo"), 
                   PathBuf::from("/game/objects/room.meta"));
        
        // Test with custom objects directory
        let mut custom_config = Config::default();
        custom_config.objects_directory = "custom_objects".to_string();
        assert_eq!(PathUtils::object_meta_path(&work_dir, &custom_config, "test"), 
                   PathBuf::from("/game/custom_objects/test.meta"));
    }

    #[test]
    fn test_ensure_moo_extension() {
        assert_eq!(PathUtils::ensure_moo_extension("player"), "player.moo");
        assert_eq!(PathUtils::ensure_moo_extension("room.moo"), "room.moo");
        assert_eq!(PathUtils::ensure_moo_extension("test"), "test.moo");
        assert_eq!(PathUtils::ensure_moo_extension(""), ".moo");
    }
}
