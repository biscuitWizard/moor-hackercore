use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Configuration for a MOO object file, stored in .meta files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    /// List of property names to ignore when processing the object
    pub ignored_properties: Option<HashSet<String>>,
    
    /// List of verb names to ignore when processing the object
    pub ignored_verbs: Option<HashSet<String>>,
    
    /// Additional metadata about the object
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            ignored_properties: None,
            ignored_verbs: None,
            metadata: None,
        }
    }
}

impl MetaConfig {
    /// Create a new empty MetaConfig
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load MetaConfig from a .meta file
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.into();
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(&path)?;
        let config: MetaConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save MetaConfig to a .meta file
    pub fn to_file<P: Into<PathBuf>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.into();
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
    
    /// Check if a property should be ignored
    pub fn is_property_ignored(&self, property_name: &str) -> bool {
        self.ignored_properties
            .as_ref()
            .map(|props| props.contains(property_name))
            .unwrap_or(false)
    }
    
    /// Check if a verb should be ignored
    pub fn is_verb_ignored(&self, verb_name: &str) -> bool {
        self.ignored_verbs
            .as_ref()
            .map(|verbs| verbs.contains(verb_name))
            .unwrap_or(false)
    }
    
    /// Add a property to the ignored list
    pub fn ignore_property(&mut self, property_name: String) {
        self.ignored_properties
            .get_or_insert_with(HashSet::new)
            .insert(property_name);
    }
    
    /// Add a verb to the ignored list
    pub fn ignore_verb(&mut self, verb_name: String) {
        self.ignored_verbs
            .get_or_insert_with(HashSet::new)
            .insert(verb_name);
    }
}
