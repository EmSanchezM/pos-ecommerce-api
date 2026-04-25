//! FiscalSequence entity - manages sequential invoice numbering per terminal

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::FiscalError;
use crate::domain::value_objects::FiscalSequenceId;
use identity::StoreId;
use pos_core::TerminalId;

/// FiscalSequence entity managing sequential invoice numbering per terminal.
///
/// Invariants:
/// - current_number must be between range_start and range_end
/// - Once the range is exhausted, no more numbers can be generated
/// - Prefix is used to construct the full invoice number (e.g., "001-001-01-" + number)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalSequence {
    id: FiscalSequenceId,
    store_id: StoreId,
    terminal_id: TerminalId,
    cai_range_id: uuid::Uuid,
    prefix: String,
    current_number: i64,
    range_start: i64,
    range_end: i64,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FiscalSequence {
    /// Creates a new FiscalSequence
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        terminal_id: TerminalId,
        cai_range_id: uuid::Uuid,
        prefix: String,
        range_start: i64,
        range_end: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: FiscalSequenceId::new(),
            store_id,
            terminal_id,
            cai_range_id,
            prefix,
            current_number: range_start - 1,
            range_start,
            range_end,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a FiscalSequence from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: FiscalSequenceId,
        store_id: StoreId,
        terminal_id: TerminalId,
        cai_range_id: uuid::Uuid,
        prefix: String,
        current_number: i64,
        range_start: i64,
        range_end: i64,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            terminal_id,
            cai_range_id,
            prefix,
            current_number,
            range_start,
            range_end,
            is_active,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Business Methods
    // =========================================================================

    /// Generates the next invoice number in the sequence.
    ///
    /// Returns the formatted invoice number (prefix + zero-padded number)
    /// or an error if the range is exhausted.
    pub fn next_number(&mut self) -> Result<String, FiscalError> {
        let next = self.current_number + 1;
        if next > self.range_end {
            return Err(FiscalError::SequenceExhausted);
        }

        self.current_number = next;
        self.updated_at = Utc::now();

        Ok(format!("{}{:08}", self.prefix, next))
    }

    /// Returns true if the sequence range is fully exhausted
    pub fn is_exhausted(&self) -> bool {
        self.current_number >= self.range_end
    }

    /// Returns the number of remaining invoice numbers in the range
    pub fn remaining(&self) -> i64 {
        (self.range_end - self.current_number).max(0)
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> FiscalSequenceId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    pub fn cai_range_id(&self) -> uuid::Uuid {
        self.cai_range_id
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn current_number(&self) -> i64 {
        self.current_number
    }

    pub fn range_start(&self) -> i64 {
        self.range_start
    }

    pub fn range_end(&self) -> i64 {
        self.range_end
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sequence() -> FiscalSequence {
        FiscalSequence::create(
            StoreId::new(),
            TerminalId::new(),
            uuid::Uuid::now_v7(),
            "001-001-01-".to_string(),
            1,
            100,
        )
    }

    #[test]
    fn test_create_sequence() {
        let seq = create_test_sequence();

        assert!(seq.is_active());
        assert!(!seq.is_exhausted());
        assert_eq!(seq.remaining(), 100);
        assert_eq!(seq.range_start(), 1);
        assert_eq!(seq.range_end(), 100);
    }

    #[test]
    fn test_next_number() {
        let mut seq = create_test_sequence();

        let number = seq.next_number().unwrap();
        assert_eq!(number, "001-001-01-00000001");
        assert_eq!(seq.remaining(), 99);

        let number = seq.next_number().unwrap();
        assert_eq!(number, "001-001-01-00000002");
        assert_eq!(seq.remaining(), 98);
    }

    #[test]
    fn test_sequence_exhaustion() {
        let mut seq = FiscalSequence::create(
            StoreId::new(),
            TerminalId::new(),
            uuid::Uuid::now_v7(),
            "001-001-01-".to_string(),
            1,
            2,
        );

        assert_eq!(seq.remaining(), 2);

        seq.next_number().unwrap();
        assert_eq!(seq.remaining(), 1);
        assert!(!seq.is_exhausted());

        seq.next_number().unwrap();
        assert_eq!(seq.remaining(), 0);
        assert!(seq.is_exhausted());

        let result = seq.next_number();
        assert!(result.is_err());
    }
}
