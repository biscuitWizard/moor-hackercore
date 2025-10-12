// Public modules for integration tests and library usage
pub mod config;
pub mod database;
pub mod object_diff;
pub mod operations;
pub mod providers;
pub mod router;
pub mod types;
pub mod util;

// Re-export commonly used types for convenience
pub use config::Config;
pub use database::{Database, DatabaseRef};
pub use operations::{OperationRegistry, create_default_registry, create_registry_with_config};
pub use router::{create_http_router, start_http_server};
