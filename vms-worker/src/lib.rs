#![recursion_limit = "256"]

pub mod config;
pub mod git_ops;
pub mod meta_config;
pub mod vms;
pub use config::Config;
pub use git_ops::GitRepository;
pub use meta_config::MetaConfig;
pub use vms::{VmsOperation, VmsProcessor};
pub use moor_objdef::{ObjectDefinitionLoader, dump_object, collect_object_definitions};

// Re-export moor types for convenience
pub use moor_compiler::{ObjectDefinition, ObjVerbDef, ObjPropDef, ObjPropOverride};
pub use moor_var::Obj;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_meta_config() {
        let mut config = MetaConfig::new();
        
        config.ignore_property("ignored_prop".to_string());
        config.ignore_verb("ignored_verb".to_string());
        
        assert!(config.is_property_ignored("ignored_prop"));
        assert!(config.is_verb_ignored("ignored_verb"));
        assert!(!config.is_property_ignored("normal_prop"));
        assert!(!config.is_verb_ignored("normal_verb"));
    }
}