use fjall::Partition;
use moor_var::Obj;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::{ProviderError, ProviderResult};
use crate::types::{Permission, User};

/// Represents the user storage as a HashMap where key is user ID and value is User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStorage {
    pub users: std::collections::HashMap<String, User>,
}

impl Default for UserStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStorage {
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }
}

/// Provider trait for user management
pub trait UserProvider: Send + Sync {
    /// Create a new user
    fn create_user(&self, id: String, email: String, v_obj: Obj) -> ProviderResult<User>;

    /// Get a user by ID
    fn get_user(&self, user_id: &str) -> ProviderResult<Option<User>>;

    /// Get a user by email
    fn get_user_by_email(&self, email: &str) -> ProviderResult<Option<User>>;

    /// Get a user by v_obj
    fn get_user_by_v_obj(&self, v_obj: Obj) -> ProviderResult<Option<User>>;

    /// Update an existing user
    fn update_user(&self, user: &User) -> ProviderResult<()>;

    /// Delete a user by ID
    fn delete_user(&self, user_id: &str) -> ProviderResult<bool>;

    /// List all users
    fn list_users(&self) -> ProviderResult<Vec<User>>;

    /// Add an authorized key to a user
    fn add_authorized_key(&self, user_id: &str, key: String) -> ProviderResult<()>;

    /// Remove an authorized key from a user
    fn remove_authorized_key(&self, user_id: &str, key: &str) -> ProviderResult<bool>;

    /// Add a permission to a user
    fn add_permission(&self, user_id: &str, permission: Permission) -> ProviderResult<()>;

    /// Remove a permission from a user
    fn remove_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool>;

    /// Check if a user has a specific permission
    fn has_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool>;

    /// Get the default "Everyone" user (for unauthenticated users)
    fn get_everyone_user(&self) -> ProviderResult<User>;

    /// Ensure the default "Everyone" user exists
    fn ensure_everyone_user(&self) -> ProviderResult<()>;

    /// Get the default "Wizard" user (system admin with all permissions)
    fn get_wizard_user(&self) -> ProviderResult<User>;

    /// Ensure the default "Wizard" user exists with the given API key
    fn ensure_wizard_user(&self, api_key: String) -> ProviderResult<()>;

    /// Disable a user (prevents authentication)
    fn disable_user(&self, user_id: &str) -> ProviderResult<()>;

    /// Enable a user (allows authentication)
    fn enable_user(&self, user_id: &str) -> ProviderResult<()>;

    /// Check if a user is disabled
    fn is_disabled(&self, user_id: &str) -> ProviderResult<bool>;

    /// Generate a new API key for a user and return it
    fn generate_api_key(&self, user_id: &str) -> ProviderResult<String>;

    /// Delete an API key from a user (alias for remove_authorized_key but returns the key if found)
    fn delete_api_key(&self, user_id: &str, key: &str) -> ProviderResult<bool>;
}

/// Implementation of UserProvider using Fjall
pub struct UserProviderImpl {
    users_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl UserProviderImpl {
    /// Create a new user provider
    pub fn new(users_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            users_tree,
            flush_sender,
        }
    }

    /// Load the user storage from the database
    fn load_user_storage(&self) -> ProviderResult<UserStorage> {
        match self.users_tree.get(b"user_storage")? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let storage: UserStorage = serde_json::from_str(&json).map_err(|e| {
                    ProviderError::SerializationError(format!("JSON parse error: {e}"))
                })?;
                Ok(storage)
            }
            None => Ok(UserStorage::new()),
        }
    }

    /// Save the user storage to the database
    fn save_user_storage(&self, storage: &UserStorage) -> ProviderResult<()> {
        let json = serde_json::to_string(storage).map_err(|e| {
            ProviderError::SerializationError(format!("JSON serialization error: {e}"))
        })?;
        self.users_tree.insert(b"user_storage", json.as_bytes())?;

        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }

        Ok(())
    }

    /// Create the default "Everyone" user
    fn create_everyone_user() -> User {
        // Everyone user has no permissions by default
        User::new_system_user(
            "Everyone".to_string(),
            "everyone@system".to_string(),
            Obj::mk_id(0),
        )
    }

    /// Create the default "Wizard" user with all permissions
    fn create_wizard_user(api_key: String) -> User {
        let mut user = User::new_system_user(
            "Wizard".to_string(),
            "wizard@system".to_string(),
            Obj::mk_id(1),
        );
        // Add the API key as an authorized key
        user.add_authorized_key(api_key);
        // Add all permissions
        user.add_permission(Permission::ApproveChanges);
        user.add_permission(Permission::SubmitChanges);
        user.add_permission(Permission::Clone);
        user.add_permission(Permission::CreateUser);
        user.add_permission(Permission::DisableUser);
        user.add_permission(Permission::ManagePermissions);
        user.add_permission(Permission::ManageApiKeys);
        user
    }
}

impl UserProvider for UserProviderImpl {
    fn create_user(&self, id: String, email: String, v_obj: Obj) -> ProviderResult<User> {
        let mut storage = self.load_user_storage()?;

        // Check if user already exists
        if storage.users.contains_key(&id) {
            return Err(ProviderError::InvalidOperation(format!(
                "User '{id}' already exists"
            )));
        }

        // Check if email is already taken
        if storage.users.values().any(|u| u.email == email) {
            return Err(ProviderError::InvalidOperation(format!(
                "Email '{email}' is already taken"
            )));
        }

        // Check if v_obj is already taken
        if storage.users.values().any(|u| u.v_obj == v_obj) {
            return Err(ProviderError::InvalidOperation(format!(
                "v_obj {v_obj:?} is already taken"
            )));
        }

        let user = User::new(id.clone(), email, v_obj);
        storage.users.insert(id.clone(), user.clone());

        self.save_user_storage(&storage)?;

        info!(
            "Created user '{}' with email '{}' and v_obj {:?}",
            user.id, user.email, user.v_obj
        );
        Ok(user)
    }

    fn get_user(&self, user_id: &str) -> ProviderResult<Option<User>> {
        let storage = self.load_user_storage()?;
        Ok(storage.users.get(user_id).cloned())
    }

    fn get_user_by_email(&self, email: &str) -> ProviderResult<Option<User>> {
        let storage = self.load_user_storage()?;
        Ok(storage.users.values().find(|u| u.email == email).cloned())
    }

    fn get_user_by_v_obj(&self, v_obj: Obj) -> ProviderResult<Option<User>> {
        let storage = self.load_user_storage()?;
        Ok(storage.users.values().find(|u| u.v_obj == v_obj).cloned())
    }

    fn update_user(&self, user: &User) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        if !storage.users.contains_key(&user.id) {
            return Err(ProviderError::InvalidOperation(format!(
                "User '{}' not found",
                user.id
            )));
        }

        storage.users.insert(user.id.clone(), user.clone());
        self.save_user_storage(&storage)?;

        debug!("Updated user '{}'", user.id);
        Ok(())
    }

    fn delete_user(&self, user_id: &str) -> ProviderResult<bool> {
        let mut storage = self.load_user_storage()?;

        // Check if the user is a system user before deletion
        if let Some(user) = storage.users.get(user_id) {
            if user.is_system_user {
                return Err(ProviderError::InvalidOperation(format!(
                    "Cannot delete system user '{user_id}'"
                )));
            }
        }

        let removed = storage.users.remove(user_id).is_some();
        if removed {
            self.save_user_storage(&storage)?;
            info!("Deleted user '{}'", user_id);
        }

        Ok(removed)
    }

    fn list_users(&self) -> ProviderResult<Vec<User>> {
        let storage = self.load_user_storage()?;
        Ok(storage.users.values().cloned().collect())
    }

    fn add_authorized_key(&self, user_id: &str, key: String) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        user.add_authorized_key(key.clone());
        self.save_user_storage(&storage)?;

        debug!("Added authorized key to user '{}'", user_id);
        Ok(())
    }

    fn remove_authorized_key(&self, user_id: &str, key: &str) -> ProviderResult<bool> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        let removed = user.remove_authorized_key(key);
        if removed {
            self.save_user_storage(&storage)?;
            debug!("Removed authorized key from user '{}'", user_id);
        }

        Ok(removed)
    }

    fn add_permission(&self, user_id: &str, permission: Permission) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        user.add_permission(permission.clone());
        self.save_user_storage(&storage)?;

        debug!("Added permission {:?} to user '{}'", permission, user_id);
        Ok(())
    }

    fn remove_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        let removed = user.remove_permission(permission);
        if removed {
            self.save_user_storage(&storage)?;
            debug!(
                "Removed permission {:?} from user '{}'",
                permission, user_id
            );
        }

        Ok(removed)
    }

    fn has_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool> {
        let storage = self.load_user_storage()?;

        let user = storage.users.get(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        Ok(user.has_permission(permission))
    }

    fn get_everyone_user(&self) -> ProviderResult<User> {
        self.get_user("Everyone").and_then(|user| {
            user.ok_or_else(|| {
                ProviderError::InvalidOperation("Everyone user not found".to_string())
            })
        })
    }

    fn ensure_everyone_user(&self) -> ProviderResult<()> {
        let storage = self.load_user_storage()?;

        if !storage.users.contains_key("Everyone") {
            let everyone_user = Self::create_everyone_user();
            let mut new_storage = storage;
            new_storage
                .users
                .insert("Everyone".to_string(), everyone_user);
            self.save_user_storage(&new_storage)?;
            info!("Created default 'Everyone' user");
        }

        Ok(())
    }

    fn get_wizard_user(&self) -> ProviderResult<User> {
        self.get_user("Wizard").and_then(|user| {
            user.ok_or_else(|| ProviderError::InvalidOperation("Wizard user not found".to_string()))
        })
    }

    fn ensure_wizard_user(&self, api_key: String) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        if !storage.users.contains_key("Wizard") {
            let wizard_user = Self::create_wizard_user(api_key.clone());
            storage.users.insert("Wizard".to_string(), wizard_user);
            self.save_user_storage(&storage)?;
            info!("Created default 'Wizard' user with all permissions");
        } else {
            // Update existing wizard user to ensure it has the correct API key
            if let Some(wizard_user) = storage.users.get_mut("Wizard") {
                // Ensure wizard has the API key if not already present
                if !wizard_user.authorized_keys.contains(&api_key) {
                    wizard_user.add_authorized_key(api_key);
                    // Also ensure all permissions are present
                    wizard_user.add_permission(Permission::ApproveChanges);
                    wizard_user.add_permission(Permission::SubmitChanges);
                    wizard_user.add_permission(Permission::Clone);
                    wizard_user.add_permission(Permission::CreateUser);
                    wizard_user.add_permission(Permission::DisableUser);
                    wizard_user.add_permission(Permission::ManagePermissions);
                    wizard_user.add_permission(Permission::ManageApiKeys);
                    // Ensure it's marked as a system user
                    wizard_user.is_system_user = true;
                    self.save_user_storage(&storage)?;
                    info!("Updated 'Wizard' user with API key and permissions");
                }
            }
        }

        Ok(())
    }

    fn disable_user(&self, user_id: &str) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        if user.is_system_user {
            return Err(ProviderError::InvalidOperation(format!(
                "Cannot disable system user '{user_id}'"
            )));
        }

        user.is_disabled = true;
        self.save_user_storage(&storage)?;

        info!("Disabled user '{}'", user_id);
        Ok(())
    }

    fn enable_user(&self, user_id: &str) -> ProviderResult<()> {
        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        user.is_disabled = false;
        self.save_user_storage(&storage)?;

        info!("Enabled user '{}'", user_id);
        Ok(())
    }

    fn is_disabled(&self, user_id: &str) -> ProviderResult<bool> {
        let storage = self.load_user_storage()?;

        let user = storage.users.get(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        Ok(user.is_disabled)
    }

    fn generate_api_key(&self, user_id: &str) -> ProviderResult<String> {
        use uuid::Uuid;

        let mut storage = self.load_user_storage()?;

        let user = storage.users.get_mut(user_id).ok_or_else(|| {
            ProviderError::InvalidOperation(format!("User '{user_id}' not found"))
        })?;

        let api_key = Uuid::new_v4().to_string();
        user.add_authorized_key(api_key.clone());
        self.save_user_storage(&storage)?;

        info!("Generated new API key for user '{}'", user_id);
        Ok(api_key)
    }

    fn delete_api_key(&self, user_id: &str, key: &str) -> ProviderResult<bool> {
        self.remove_authorized_key(user_id, key)
    }
}

// Helper trait extension for Arc wrapping
impl<T: UserProvider> UserProvider for std::sync::Arc<T> {
    fn create_user(&self, id: String, email: String, v_obj: Obj) -> ProviderResult<User> {
        (**self).create_user(id, email, v_obj)
    }

    fn get_user(&self, user_id: &str) -> ProviderResult<Option<User>> {
        (**self).get_user(user_id)
    }

    fn get_user_by_email(&self, email: &str) -> ProviderResult<Option<User>> {
        (**self).get_user_by_email(email)
    }

    fn get_user_by_v_obj(&self, v_obj: Obj) -> ProviderResult<Option<User>> {
        (**self).get_user_by_v_obj(v_obj)
    }

    fn update_user(&self, user: &User) -> ProviderResult<()> {
        (**self).update_user(user)
    }

    fn delete_user(&self, user_id: &str) -> ProviderResult<bool> {
        (**self).delete_user(user_id)
    }

    fn list_users(&self) -> ProviderResult<Vec<User>> {
        (**self).list_users()
    }

    fn add_authorized_key(&self, user_id: &str, key: String) -> ProviderResult<()> {
        (**self).add_authorized_key(user_id, key)
    }

    fn remove_authorized_key(&self, user_id: &str, key: &str) -> ProviderResult<bool> {
        (**self).remove_authorized_key(user_id, key)
    }

    fn add_permission(&self, user_id: &str, permission: Permission) -> ProviderResult<()> {
        (**self).add_permission(user_id, permission)
    }

    fn remove_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool> {
        (**self).remove_permission(user_id, permission)
    }

    fn has_permission(&self, user_id: &str, permission: &Permission) -> ProviderResult<bool> {
        (**self).has_permission(user_id, permission)
    }

    fn get_everyone_user(&self) -> ProviderResult<User> {
        (**self).get_everyone_user()
    }

    fn ensure_everyone_user(&self) -> ProviderResult<()> {
        (**self).ensure_everyone_user()
    }

    fn get_wizard_user(&self) -> ProviderResult<User> {
        (**self).get_wizard_user()
    }

    fn ensure_wizard_user(&self, api_key: String) -> ProviderResult<()> {
        (**self).ensure_wizard_user(api_key)
    }

    fn disable_user(&self, user_id: &str) -> ProviderResult<()> {
        (**self).disable_user(user_id)
    }

    fn enable_user(&self, user_id: &str) -> ProviderResult<()> {
        (**self).enable_user(user_id)
    }

    fn is_disabled(&self, user_id: &str) -> ProviderResult<bool> {
        (**self).is_disabled(user_id)
    }

    fn generate_api_key(&self, user_id: &str) -> ProviderResult<String> {
        (**self).generate_api_key(user_id)
    }

    fn delete_api_key(&self, user_id: &str, key: &str) -> ProviderResult<bool> {
        (**self).delete_api_key(user_id, key)
    }
}
