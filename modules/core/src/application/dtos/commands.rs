// Command DTOs for Store and Terminal Management
//
// These DTOs represent the input data for use case operations.
// They are framework-agnostic and can be used with any HTTP framework.

use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new terminal for a store
///
/// Requirements: 2.1, 2.2
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTerminalCommand {
    /// The store ID where the terminal will be created
    pub store_id: Uuid,
    /// Unique code for the terminal within the store (alphanumeric, 3-20 chars)
    pub code: String,
    /// Human-readable name for the terminal
    pub name: String,
}

/// Command to assign a CAI range to a terminal
///
/// Requirements: 2.2
#[derive(Debug, Clone, Deserialize)]
pub struct AssignCaiCommand {
    /// The terminal ID to assign the CAI to
    pub terminal_id: Uuid,
    /// The CAI number from the fiscal authority
    pub cai_number: String,
    /// Starting invoice number in the range
    pub range_start: i64,
    /// Ending invoice number in the range
    pub range_end: i64,
    /// Expiration date of the CAI
    pub expiration_date: NaiveDate,
}

/// Command to update an existing terminal
///
/// All fields are optional - only specified fields will be updated.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTerminalCommand {
    /// New name for the terminal (if provided)
    pub name: Option<String>,
}

/// Query parameters for listing stores with pagination and filters
///
/// Requirements: 4.1
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListStoresQuery {
    /// Filter by active status (optional)
    pub is_active: Option<bool>,
    /// Filter by e-commerce type (optional)
    pub is_ecommerce: Option<bool>,
    /// Page number (1-based, defaults to 1)
    pub page: Option<u32>,
    /// Number of items per page (defaults to 20, max 100)
    pub page_size: Option<u32>,
}

/// Query parameters for listing terminals of a store
///
/// Requirements: 4.3
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListTerminalsQuery {
    /// Filter by active status (optional)
    pub is_active: Option<bool>,
    /// Page number (1-based, defaults to 1)
    pub page: Option<u32>,
    /// Number of items per page (defaults to 20, max 100)
    pub page_size: Option<u32>,
}
