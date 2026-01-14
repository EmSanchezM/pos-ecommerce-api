// Terminal entity - Represents a point of sale terminal for invoice emission

use chrono::{DateTime, Utc};
use identity::StoreId;
use serde::{Deserialize, Serialize};

use crate::domain::entities::CaiRange;
use crate::domain::value_objects::{TerminalCode, TerminalId};

/// Terminal entity for invoice emission
/// 
/// Represents a physical or virtual terminal associated with a store
/// that can emit fiscal documents using assigned CAI ranges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminal {
    id: TerminalId,
    store_id: StoreId,
    code: TerminalCode,
    name: String,
    is_active: bool,
    current_cai: Option<CaiRange>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Terminal {
    /// Creates a new Terminal with the given store, code, and name
    /// 
    /// The terminal is created as active by default with no CAI assigned.
    pub fn create(store_id: StoreId, code: TerminalCode, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: TerminalId::new(),
            store_id,
            code,
            name,
            is_active: true,
            current_cai: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstructs a Terminal from persisted data
    pub fn reconstitute(
        id: TerminalId,
        store_id: StoreId,
        code: TerminalCode,
        name: String,
        is_active: bool,
        current_cai: Option<CaiRange>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            code,
            name,
            is_active,
            current_cai,
            created_at,
            updated_at,
        }
    }

    /// Activates the terminal
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates the terminal (preserves CAI history)
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Updates the terminal name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Assigns a new CAI range to the terminal
    pub fn set_cai(&mut self, cai: CaiRange) {
        self.current_cai = Some(cai);
        self.updated_at = Utc::now();
    }

    /// Clears the current CAI (used when CAI is exhausted or expired)
    pub fn clear_cai(&mut self) {
        self.current_cai = None;
        self.updated_at = Utc::now();
    }

    // Getters
    pub fn id(&self) -> TerminalId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn code(&self) -> &TerminalCode {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn current_cai(&self) -> Option<&CaiRange> {
        self.current_cai.as_ref()
    }

    pub fn current_cai_mut(&mut self) -> Option<&mut CaiRange> {
        self.current_cai.as_mut()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
