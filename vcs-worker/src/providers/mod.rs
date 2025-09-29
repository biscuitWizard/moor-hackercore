//! Provider pattern implementations for different aspects of the VCS database
//! 
//! Each provider focuses on a specific concern:
//! - ObjectsProvider: Pure CRUD operations for object content
//! - RefsProvider: Object name + version resolution to SHA256
//! - IndexProvider: Ordered change management and current working change tracking
//! - WorkspaceProvider: Changes that aren't yet on index (review/approval queue, idle changes)

pub mod objects;
pub mod refs;
pub mod index;
pub mod workspace;

pub mod error;

pub use error::{ProviderError, ProviderResult};
pub use objects::{ObjectsProvider, ObjectsProviderImpl};
pub use refs::RefsProviderImpl;
pub use index::IndexProviderImpl;
pub use workspace::WorkspaceProviderImpl;
