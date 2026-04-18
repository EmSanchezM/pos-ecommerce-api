// Response DTOs for identity operations

use serde::{Deserialize, Serialize};

// =============================================================================
// List Response (Simple)
// =============================================================================

/// Generic list response wrapper for endpoints that don't need full pagination
/// Use this for collections that are typically small (e.g., permissions, roles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    /// The collection items
    pub items: Vec<T>,
    /// Total number of items
    pub total: i64,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>) -> Self {
        let total = items.len() as i64;
        Self { items, total }
    }
}
