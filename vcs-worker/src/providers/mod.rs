//! Provider pattern implementations for different aspects of the VCS database
//! 
//! Each provider focuses on a specific concern:
//! - ObjectsProvider: Pure CRUD operations for object content
//! - RefsProvider: Object name + version resolution to SHA256
//! - ChangesProvider: Change tracking and metadata
//! - IndexProvider: Ordered change management and current working change tracking
//! - RepositoryProvider: Repository-level metadata and configuration

pub mod objects;
pub mod refs;
pub mod changes;
pub mod index;
pub mod repository;

pub mod error;

pub use error::{ProviderError, ProviderResult};
pub use objects::{ObjectsProvider, ObjectsProviderImpl};
pub use refs::{RefsProvider, RefsProviderImpl};
pub use changes::{ChangesProvider, ChangesProviderImpl};
pub use index::{IndexProvider, IndexProviderImpl};
pub use repository::{RepositoryProvider, RepositoryProviderImpl};
