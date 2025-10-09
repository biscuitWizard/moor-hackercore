// Public modules for integration tests and library usage
pub mod operations;
pub mod router;
pub mod config;
pub mod database;
pub mod providers;
pub mod types;
pub mod util;
pub mod object_diff;

// Re-export commonly used types for convenience
pub use operations::{create_default_registry, create_registry_with_config, OperationRegistry};
pub use router::{create_http_router, start_http_server};
pub use config::Config;
pub use database::{Database, DatabaseRef};

