// Response DTOs for Store and Terminal Management
//
// These DTOs represent the output data from use case operations.
// They are framework-agnostic and can be serialized to JSON for HTTP responses.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{CaiRange, Terminal};

/// Response DTO for terminal information
///
/// Requirements: 4.3, 4.4
#[derive(Debug, Clone, Serialize)]
pub struct TerminalResponse {
    /// Unique identifier of the terminal
    pub id: Uuid,
    /// Store ID the terminal belongs to
    pub store_id: Uuid,
    /// Unique code within the store
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Whether the terminal is active
    pub is_active: bool,
    /// Current CAI status (if assigned)
    pub cai_status: Option<CaiStatusResponse>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<Terminal> for TerminalResponse {
    fn from(terminal: Terminal) -> Self {
        let cai_status = terminal.current_cai().map(CaiStatusResponse::from);
        Self {
            id: terminal.id().into_uuid(),
            store_id: terminal.store_id().into_uuid(),
            code: terminal.code().as_str().to_string(),
            name: terminal.name().to_string(),
            is_active: terminal.is_active(),
            cai_status,
            created_at: terminal.created_at(),
            updated_at: terminal.updated_at(),
        }
    }
}

impl From<&Terminal> for TerminalResponse {
    fn from(terminal: &Terminal) -> Self {
        let cai_status = terminal.current_cai().map(CaiStatusResponse::from);
        Self {
            id: terminal.id().into_uuid(),
            store_id: terminal.store_id().into_uuid(),
            code: terminal.code().as_str().to_string(),
            name: terminal.name().to_string(),
            is_active: terminal.is_active(),
            cai_status,
            created_at: terminal.created_at(),
            updated_at: terminal.updated_at(),
        }
    }
}

/// Response DTO for CAI status information
///
/// Requirements: 3.4, 3.5
#[derive(Debug, Clone, Serialize)]
pub struct CaiStatusResponse {
    /// The CAI number from the fiscal authority
    pub cai_number: String,
    /// Current invoice number in the sequence
    pub current_number: i64,
    /// Number of remaining invoices in the range
    pub remaining: i64,
    /// Expiration date of the CAI
    pub expiration_date: NaiveDate,
    /// Whether the range is exhausted
    pub is_exhausted: bool,
    /// Warning message if CAI expires within 30 days
    pub expiration_warning: Option<String>,
}

impl From<&CaiRange> for CaiStatusResponse {
    fn from(cai: &CaiRange) -> Self {
        let expiration_warning = if cai.expires_within_days(30) && !cai.is_expired() {
            Some(format!(
                "CAI expires on {}. Please request a new CAI range.",
                cai.expiration_date()
            ))
        } else {
            None
        };

        Self {
            cai_number: cai.cai_number().as_str().to_string(),
            current_number: cai.current_number(),
            remaining: cai.remaining(),
            expiration_date: cai.expiration_date(),
            is_exhausted: cai.is_exhausted_flag(),
            expiration_warning,
        }
    }
}

impl From<CaiRange> for CaiStatusResponse {
    fn from(cai: CaiRange) -> Self {
        CaiStatusResponse::from(&cai)
    }
}

/// Response DTO for next invoice number operation
///
/// Requirements: 3.1
#[derive(Debug, Clone, Serialize)]
pub struct NextInvoiceNumberResponse {
    /// Terminal ID that issued the number
    pub terminal_id: Uuid,
    /// CAI number associated with this invoice
    pub cai_number: String,
    /// The invoice number to use
    pub invoice_number: i64,
    /// Remaining invoices in the range after this one
    pub remaining: i64,
}

/// Response DTO for store detail with terminal count
///
/// Requirements: 4.2
#[derive(Debug, Clone, Serialize)]
pub struct StoreDetailResponse {
    /// Unique identifier of the store
    pub id: Uuid,
    /// Store name
    pub name: String,
    /// Store address
    pub address: String,
    /// Whether this is an e-commerce store
    pub is_ecommerce: bool,
    /// Whether the store is active
    pub is_active: bool,
    /// Count of active terminals in this store
    pub active_terminals_count: i64,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Response DTO for a store in a list (without terminal count for efficiency)
///
/// Requirements: 4.1
#[derive(Debug, Clone, Serialize)]
pub struct StoreListItemResponse {
    /// Unique identifier of the store
    pub id: Uuid,
    /// Store name
    pub name: String,
    /// Store address
    pub address: String,
    /// Whether this is an e-commerce store
    pub is_ecommerce: bool,
    /// Whether the store is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Paginated response for store listing
///
/// Requirements: 4.1
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedStoresResponse {
    /// List of stores in the current page
    pub items: Vec<StoreListItemResponse>,
    /// Total number of stores matching the filters
    pub total: i64,
    /// Current page number (1-based)
    pub page: u32,
    /// Number of items per page
    pub page_size: u32,
    /// Total number of pages
    pub total_pages: u32,
}

/// Paginated response for terminal listing
///
/// Requirements: 4.3
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedTerminalsResponse {
    /// List of terminals in the current page
    pub items: Vec<TerminalResponse>,
    /// Total number of terminals matching the filters
    pub total: i64,
    /// Current page number (1-based)
    pub page: u32,
    /// Number of items per page
    pub page_size: u32,
    /// Total number of pages
    pub total_pages: u32,
}

/// Response DTO for terminal detail with CAI history
///
/// Requirements: 4.4
#[derive(Debug, Clone, Serialize)]
pub struct TerminalDetailResponse {
    /// Unique identifier of the terminal
    pub id: Uuid,
    /// Store ID the terminal belongs to
    pub store_id: Uuid,
    /// Unique code within the store
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Whether the terminal is active
    pub is_active: bool,
    /// Current CAI status (if assigned)
    pub cai_status: Option<CaiStatusResponse>,
    /// Complete history of CAI ranges
    pub cai_history: Vec<CaiHistoryItemResponse>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Response DTO for a CAI range in the history
///
/// Requirements: 4.4
#[derive(Debug, Clone, Serialize)]
pub struct CaiHistoryItemResponse {
    /// Unique identifier of the CAI range
    pub id: Uuid,
    /// The CAI number from the fiscal authority
    pub cai_number: String,
    /// Starting invoice number in the range
    pub range_start: i64,
    /// Ending invoice number in the range
    pub range_end: i64,
    /// Current invoice number in the sequence
    pub current_number: i64,
    /// Expiration date of the CAI
    pub expiration_date: NaiveDate,
    /// Whether the range is exhausted
    pub is_exhausted: bool,
    /// Whether the CAI is expired
    pub is_expired: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<&CaiRange> for CaiHistoryItemResponse {
    fn from(cai: &CaiRange) -> Self {
        Self {
            id: cai.id(),
            cai_number: cai.cai_number().as_str().to_string(),
            range_start: cai.range_start(),
            range_end: cai.range_end(),
            current_number: cai.current_number(),
            expiration_date: cai.expiration_date(),
            is_exhausted: cai.is_exhausted_flag(),
            is_expired: cai.is_expired(),
            created_at: cai.created_at(),
        }
    }
}

impl From<CaiRange> for CaiHistoryItemResponse {
    fn from(cai: CaiRange) -> Self {
        CaiHistoryItemResponse::from(&cai)
    }
}
