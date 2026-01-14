// StoreRepository trait - interface for store persistence

use async_trait::async_trait;

use crate::domain::entities::{Store, User};
use crate::domain::value_objects::StoreId;
use crate::error::IdentityError;

/// Repository trait for Store entity persistence
///
/// Defines the contract for storing and retrieving stores,
/// as well as querying store membership.
#[async_trait]
pub trait StoreRepository: Send + Sync {
    /// Saves a new store to the repository
    ///
    /// # Errors
    /// - `IdentityError::Database` on database errors
    async fn save(&self, store: &Store) -> Result<(), IdentityError>;

    /// Finds a store by its ID
    ///
    /// Returns `None` if no store with the given ID exists.
    async fn find_by_id(&self, id: StoreId) -> Result<Option<Store>, IdentityError>;

    /// Returns all stores in the repository
    async fn find_all(&self) -> Result<Vec<Store>, IdentityError>;

    /// Returns all active stores
    async fn find_active(&self) -> Result<Vec<Store>, IdentityError>;

    /// Updates an existing store
    ///
    /// # Errors
    /// - `IdentityError::StoreNotFound` if the store doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn update(&self, store: &Store) -> Result<(), IdentityError>;

    /// Gets all users assigned to a store
    async fn get_users(&self, store_id: StoreId) -> Result<Vec<User>, IdentityError>;
}
