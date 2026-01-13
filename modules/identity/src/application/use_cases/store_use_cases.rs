// Store use cases - Application layer business logic for store management
//
// Requirements: 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3

use std::sync::Arc;

use crate::application::dtos::{CreateStoreCommand, UpdateStoreCommand};
use crate::domain::entities::{AuditAction, AuditEntry, Store};
use crate::domain::repositories::{AuditRepository, StoreRepository, UserRepository};
use crate::domain::value_objects::{StoreId, UserId};
use crate::error::IdentityError;

// =============================================================================
// CreateStoreUseCase
// =============================================================================

/// Use case for creating a new store
///
/// Creates a store with the provided details. If is_ecommerce is not specified,
/// it defaults to false (physical POS store).
///
/// Requirements: 7.1, 7.2
pub struct CreateStoreUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<S, A> CreateStoreUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreateStoreUseCase
    pub fn new(store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new store
    ///
    /// # Arguments
    /// * `command` - The create store command containing store data
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The created Store on success
    ///
    /// # Notes
    /// - is_ecommerce defaults to false if not specified (Requirement 7.2)
    pub async fn execute(
        &self,
        command: CreateStoreCommand,
        actor_id: UserId,
    ) -> Result<Store, IdentityError> {
        // Create store - is_ecommerce defaults to false in CreateStoreCommand via #[serde(default)]
        // Requirement 7.2: default is_ecommerce to false
        let store = if command.is_ecommerce {
            Store::create_ecommerce(command.name, command.address)
        } else {
            Store::create(command.name, command.address)
        };

        // Save to repository (Requirement 7.1)
        self.store_repo.save(&store).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_create(
            "store",
            store.id().into_uuid(),
            &store,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(store)
    }
}


// =============================================================================
// UpdateStoreUseCase
// =============================================================================

/// Use case for updating an existing store's details
///
/// Updates the store's name, address, and/or is_ecommerce flag.
///
/// Requirements: 7.4
pub struct UpdateStoreUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<S, A> UpdateStoreUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateStoreUseCase
    pub fn new(store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update a store's details
    ///
    /// # Arguments
    /// * `store_id` - ID of the store to update
    /// * `command` - The update store command containing fields to update
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The updated Store on success
    ///
    /// # Errors
    /// * `IdentityError::StoreNotFound` - If store doesn't exist
    pub async fn execute(
        &self,
        store_id: StoreId,
        command: UpdateStoreCommand,
        actor_id: UserId,
    ) -> Result<Store, IdentityError> {
        // Find the store
        let mut store = self
            .store_repo
            .find_by_id(store_id)
            .await?
            .ok_or(IdentityError::StoreNotFound(store_id.into_uuid()))?;

        // Store old state for audit
        let old_store = store.clone();

        // Update name if provided (Requirement 7.4)
        if let Some(name) = command.name {
            store.set_name(name);
        }

        // Update address if provided (Requirement 7.4)
        if let Some(address) = command.address {
            store.set_address(address);
        }

        // Update is_ecommerce if provided (Requirement 7.4)
        if let Some(is_ecommerce) = command.is_ecommerce {
            store.set_ecommerce(is_ecommerce);
        }

        // Save updated store
        self.store_repo.update(&store).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_update(
            "store",
            store_id.into_uuid(),
            &old_store,
            &store,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(store)
    }
}


// =============================================================================
// SetStoreActiveUseCase
// =============================================================================

/// Use case for enabling or disabling a store
///
/// Updates the store's is_active flag and creates an audit entry.
///
/// Requirements: 7.3
pub struct SetStoreActiveUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<S, A> SetStoreActiveUseCase<S, A>
where
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of SetStoreActiveUseCase
    pub fn new(store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to set a store's active status
    ///
    /// # Arguments
    /// * `store_id` - ID of the store to update
    /// * `is_active` - Whether the store should be active
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The updated Store on success
    ///
    /// # Errors
    /// * `IdentityError::StoreNotFound` - If store doesn't exist
    pub async fn execute(
        &self,
        store_id: StoreId,
        is_active: bool,
        actor_id: UserId,
    ) -> Result<Store, IdentityError> {
        // Find the store
        let mut store = self
            .store_repo
            .find_by_id(store_id)
            .await?
            .ok_or(IdentityError::StoreNotFound(store_id.into_uuid()))?;

        // Store old state for audit
        let old_store = store.clone();

        // Update active status (Requirement 7.3)
        if is_active {
            store.activate();
        } else {
            store.deactivate();
        }

        // Save updated store
        self.store_repo.update(&store).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_update(
            "store",
            store_id.into_uuid(),
            &old_store,
            &store,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(store)
    }
}


// =============================================================================
// AddUserToStoreUseCase
// =============================================================================

/// Use case for adding a user to a store
///
/// Creates a membership relationship between a user and a store,
/// allowing the user to be assigned roles in that store.
///
/// Requirements: 8.1, 8.2
pub struct AddUserToStoreUseCase<U, S, A>
where
    U: UserRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<U, S, A> AddUserToStoreUseCase<U, S, A>
where
    U: UserRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of AddUserToStoreUseCase
    pub fn new(user_repo: Arc<U>, store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to add a user to a store
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to add
    /// * `store_id` - ID of the store to add the user to
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    /// * `IdentityError::StoreNotFound` - If store doesn't exist
    pub async fn execute(
        &self,
        user_id: UserId,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Verify store exists
        let store = self
            .store_repo
            .find_by_id(store_id)
            .await?
            .ok_or(IdentityError::StoreNotFound(store_id.into_uuid()))?;

        // Add user to store (Requirements 8.1, 8.2)
        self.user_repo.add_to_store(user_id, store_id).await?;

        // Create audit entry (Requirement 9.4)
        let audit_entry = AuditEntry::create(
            "user_store".to_string(),
            user_id.into_uuid(),
            AuditAction::UserAddedToStore,
            None,
            Some(serde_json::json!({
                "user_id": user_id.into_uuid(),
                "username": user.username().as_str(),
                "store_id": store_id.into_uuid(),
                "store_name": store.name()
            })),
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// RemoveUserFromStoreUseCase
// =============================================================================

/// Use case for removing a user from a store
///
/// Removes the membership relationship between a user and a store.
/// This also implicitly removes any role assignments for that user in that store.
///
/// Requirements: 8.3
pub struct RemoveUserFromStoreUseCase<U, S, A>
where
    U: UserRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<U, S, A> RemoveUserFromStoreUseCase<U, S, A>
where
    U: UserRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of RemoveUserFromStoreUseCase
    pub fn new(user_repo: Arc<U>, store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to remove a user from a store
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to remove
    /// * `store_id` - ID of the store to remove the user from
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    pub async fn execute(
        &self,
        user_id: UserId,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Get store info for audit (if it exists)
        let store_name = self
            .store_repo
            .find_by_id(store_id)
            .await?
            .map(|s| s.name().to_string());

        // Remove user from store (Requirement 8.3)
        self.user_repo.remove_from_store(user_id, store_id).await?;

        // Create audit entry (Requirement 9.4)
        let audit_entry = AuditEntry::create(
            "user_store".to_string(),
            user_id.into_uuid(),
            AuditAction::UserRemovedFromStore,
            Some(serde_json::json!({
                "user_id": user_id.into_uuid(),
                "username": user.username().as_str(),
                "store_id": store_id.into_uuid(),
                "store_name": store_name
            })),
            None,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}
