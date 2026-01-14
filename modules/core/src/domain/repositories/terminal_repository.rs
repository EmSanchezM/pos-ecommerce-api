// TerminalRepository trait - Repository port for Terminal persistence
// Requirements: 5.2

use async_trait::async_trait;
use identity::StoreId;

use crate::domain::entities::{CaiRange, Terminal};
use crate::domain::value_objects::{TerminalCode, TerminalId};
use crate::error::CoreError;

/// Repository trait (port) for Terminal persistence
/// 
/// This trait defines the contract for terminal data access operations.
/// Implementations (adapters) can use any persistence mechanism (PostgreSQL, etc.)
#[async_trait]
pub trait TerminalRepository: Send + Sync {
    /// Saves a new terminal to the repository
    /// 
    /// # Arguments
    /// * `terminal` - The terminal entity to save
    /// 
    /// # Returns
    /// * `Ok(())` - If the terminal was saved successfully
    /// * `Err(CoreError)` - If there was an error saving the terminal
    async fn save(&self, terminal: &Terminal) -> Result<(), CoreError>;

    /// Finds a terminal by its unique identifier
    /// 
    /// # Arguments
    /// * `id` - The terminal ID to search for
    /// 
    /// # Returns
    /// * `Ok(Some(Terminal))` - If the terminal was found
    /// * `Ok(None)` - If no terminal exists with the given ID
    /// * `Err(CoreError)` - If there was a database error
    async fn find_by_id(&self, id: TerminalId) -> Result<Option<Terminal>, CoreError>;

    /// Finds all terminals belonging to a specific store
    /// 
    /// # Arguments
    /// * `store_id` - The store ID to filter by
    /// 
    /// # Returns
    /// * `Ok(Vec<Terminal>)` - List of terminals for the store (may be empty)
    /// * `Err(CoreError)` - If there was a database error
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<Terminal>, CoreError>;

    /// Finds a terminal by its code within a specific store
    /// 
    /// Terminal codes are unique within a store but not globally.
    /// 
    /// # Arguments
    /// * `store_id` - The store ID to search within
    /// * `code` - The terminal code to search for
    /// 
    /// # Returns
    /// * `Ok(Some(Terminal))` - If a terminal with the code exists in the store
    /// * `Ok(None)` - If no terminal with the code exists in the store
    /// * `Err(CoreError)` - If there was a database error
    async fn find_by_code(
        &self,
        store_id: StoreId,
        code: &TerminalCode,
    ) -> Result<Option<Terminal>, CoreError>;

    /// Updates an existing terminal in the repository
    /// 
    /// # Arguments
    /// * `terminal` - The terminal entity with updated values
    /// 
    /// # Returns
    /// * `Ok(())` - If the terminal was updated successfully
    /// * `Err(CoreError)` - If there was an error updating the terminal
    async fn update(&self, terminal: &Terminal) -> Result<(), CoreError>;

    /// Saves a CAI range for a terminal
    /// 
    /// This creates a new CAI range record associated with the terminal.
    /// The terminal's current_cai should be updated separately via `update()`.
    /// 
    /// # Arguments
    /// * `terminal_id` - The terminal to associate the CAI range with
    /// * `cai` - The CAI range to save
    /// 
    /// # Returns
    /// * `Ok(())` - If the CAI range was saved successfully
    /// * `Err(CoreError)` - If there was an error saving the CAI range
    async fn save_cai_range(
        &self,
        terminal_id: TerminalId,
        cai: &CaiRange,
    ) -> Result<(), CoreError>;

    /// Gets the complete CAI history for a terminal
    /// 
    /// Returns all CAI ranges that have been assigned to the terminal,
    /// ordered by creation date (newest first).
    /// 
    /// # Arguments
    /// * `terminal_id` - The terminal to get history for
    /// 
    /// # Returns
    /// * `Ok(Vec<CaiRange>)` - List of CAI ranges (may be empty)
    /// * `Err(CoreError)` - If there was a database error
    async fn get_cai_history(&self, terminal_id: TerminalId) -> Result<Vec<CaiRange>, CoreError>;

    /// Atomically increments and returns the next invoice number
    /// 
    /// This operation MUST be atomic to prevent duplicate invoice numbers.
    /// It increments the current_number in the active CAI range and returns
    /// the number that was used (before increment).
    /// 
    /// # Arguments
    /// * `terminal_id` - The terminal to get the next invoice number for
    /// 
    /// # Returns
    /// * `Ok(i64)` - The invoice number to use
    /// * `Err(CoreError::NoCaiAssigned)` - If no CAI is assigned
    /// * `Err(CoreError::CaiExpired)` - If the CAI has expired
    /// * `Err(CoreError::CaiRangeExhausted)` - If the range is exhausted
    /// * `Err(CoreError)` - If there was a database error
    async fn increment_and_get_invoice_number(
        &self,
        terminal_id: TerminalId,
    ) -> Result<i64, CoreError>;

    /// Counts the number of active terminals for a store
    /// 
    /// # Arguments
    /// * `store_id` - The store to count terminals for
    /// 
    /// # Returns
    /// * `Ok(i64)` - The count of active terminals
    /// * `Err(CoreError)` - If there was a database error
    async fn count_active_by_store(&self, store_id: StoreId) -> Result<i64, CoreError>;

    /// Deactivates all terminals belonging to a store
    /// 
    /// This is used when a store is deactivated to cascade the deactivation
    /// to all its terminals. CAI history is preserved.
    /// 
    /// # Arguments
    /// * `store_id` - The store whose terminals should be deactivated
    /// 
    /// # Returns
    /// * `Ok(())` - If terminals were deactivated successfully
    /// * `Err(CoreError)` - If there was a database error
    async fn deactivate_by_store(&self, store_id: StoreId) -> Result<(), CoreError>;
}
