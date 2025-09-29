//! Provider pattern implementations for different aspects of the VCS database
//! 
//! Each provider focuses on a specific concern:
//! - ObjectsProvider: Pure CRUD operations for object content
//! - RefsProvider: Object name + version resolution to SHA256
//! - HeadProvider: Working state management (list of object refs)
//! - ChangesProvider: Change tracking and metadata
//! - RepositoryProvider: Repository-level metadata and configuration

pub mod objects;
pub mod refs;
pub mod head;
pub mod changes;
pub mod repository;

pub mod error;

pub use error::{ProviderError, ProviderResult};
pub use objects::{ObjectsProvider, ObjectsProviderImpl};
pub use refs::{RefsProvider, RefsProviderImpl};
pub use head::{HeadProvider, HeadProviderImpl};
pub use changes::{ChangesProvider, ChangesProviderImpl};
pub use repository::{RepositoryProvider, RepositoryProviderImpl};
