// CaiRange entity - Manages CAI (Código de Autorización de Impresión) ranges for invoice numbering

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::CaiNumber;
use crate::error::CaiError;

/// CAI range for invoice numbering
/// 
/// Represents a range of invoice numbers authorized by the fiscal authority.
/// Each terminal can have one active CAI range at a time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaiRange {
    id: Uuid,
    cai_number: CaiNumber,
    range_start: i64,
    range_end: i64,
    current_number: i64,
    expiration_date: NaiveDate,
    is_exhausted: bool,
    created_at: DateTime<Utc>,
}

impl CaiRange {
    /// Creates a new CaiRange
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        cai_number: CaiNumber,
        range_start: i64,
        range_end: i64,
        current_number: i64,
        expiration_date: NaiveDate,
        is_exhausted: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            cai_number,
            range_start,
            range_end,
            current_number,
            expiration_date,
            is_exhausted,
            created_at,
        }
    }

    /// Returns the number of remaining invoice numbers in this range
    pub fn remaining(&self) -> i64 {
        if self.current_number > self.range_end {
            0
        } else {
            self.range_end - self.current_number + 1
        }
    }

    /// Checks if the CAI has expired based on current date
    pub fn is_expired(&self) -> bool {
        Utc::now().date_naive() > self.expiration_date
    }

    /// Checks if the CAI expires within the specified number of days
    pub fn expires_within_days(&self, days: i64) -> bool {
        let warning_date = Utc::now().date_naive() + chrono::Duration::days(days);
        self.expiration_date <= warning_date
    }

    /// Validates if the CAI can emit an invoice
    /// 
    /// # Returns
    /// * `Ok(())` - If the CAI is valid for emission
    /// * `Err(CaiError::Expired)` - If the CAI has expired
    /// * `Err(CaiError::RangeExhausted)` - If the range is exhausted
    pub fn can_emit(&self) -> Result<(), CaiError> {
        if self.is_expired() {
            return Err(CaiError::Expired);
        }
        if self.is_exhausted || self.current_number > self.range_end {
            return Err(CaiError::RangeExhausted);
        }
        Ok(())
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn cai_number(&self) -> &CaiNumber {
        &self.cai_number
    }

    pub fn range_start(&self) -> i64 {
        self.range_start
    }

    pub fn range_end(&self) -> i64 {
        self.range_end
    }

    pub fn current_number(&self) -> i64 {
        self.current_number
    }

    pub fn expiration_date(&self) -> NaiveDate {
        self.expiration_date
    }

    pub fn is_exhausted_flag(&self) -> bool {
        self.is_exhausted
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Marks the range as exhausted
    pub fn mark_exhausted(&mut self) {
        self.is_exhausted = true;
    }

    /// Increments the current number and returns the new value
    pub fn increment_current(&mut self) -> i64 {
        self.current_number += 1;
        if self.current_number > self.range_end {
            self.is_exhausted = true;
        }
        self.current_number - 1 // Return the number that was used
    }
}
