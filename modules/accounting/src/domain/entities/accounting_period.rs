//! AccountingPeriod entity — a time window (typically a month) in which entries
//! are accepted. Once closed, no more entries can be posted to it; corrections
//! require a new entry in the next open period.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AccountingError;
use crate::domain::value_objects::{AccountingPeriodId, PeriodStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingPeriod {
    id: AccountingPeriodId,
    name: String,
    fiscal_year: i32,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: PeriodStatus,
    closed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AccountingPeriod {
    pub fn create(
        name: impl Into<String>,
        fiscal_year: i32,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<Self, AccountingError> {
        if starts_at >= ends_at {
            return Err(AccountingError::InvalidPeriodRange);
        }
        let now = Utc::now();
        Ok(Self {
            id: AccountingPeriodId::new(),
            name: name.into(),
            fiscal_year,
            starts_at,
            ends_at,
            status: PeriodStatus::Open,
            closed_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AccountingPeriodId,
        name: String,
        fiscal_year: i32,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        status: PeriodStatus,
        closed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            fiscal_year,
            starts_at,
            ends_at,
            status,
            closed_at,
            created_at,
            updated_at,
        }
    }

    pub fn close(&mut self) -> Result<(), AccountingError> {
        if self.status == PeriodStatus::Closed {
            return Err(AccountingError::PeriodClosed(self.id.into_uuid()));
        }
        let now = Utc::now();
        self.status = PeriodStatus::Closed;
        self.closed_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.status == PeriodStatus::Open
    }

    pub fn id(&self) -> AccountingPeriodId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fiscal_year(&self) -> i32 {
        self.fiscal_year
    }
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }
    pub fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }
    pub fn status(&self) -> PeriodStatus {
        self.status
    }
    pub fn closed_at(&self) -> Option<DateTime<Utc>> {
        self.closed_at
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
    use chrono::TimeZone;

    fn jan() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
        )
    }

    #[test]
    fn create_starts_open() {
        let (s, e) = jan();
        let p = AccountingPeriod::create("Jan 2026", 2026, s, e).unwrap();
        assert!(p.is_open());
        assert!(p.closed_at().is_none());
    }

    #[test]
    fn invalid_range_is_rejected() {
        let (s, e) = jan();
        assert!(AccountingPeriod::create("Bad", 2026, e, s).is_err());
    }

    #[test]
    fn close_marks_closed_and_records_timestamp() {
        let (s, e) = jan();
        let mut p = AccountingPeriod::create("Jan 2026", 2026, s, e).unwrap();
        p.close().unwrap();
        assert_eq!(p.status(), PeriodStatus::Closed);
        assert!(p.closed_at().is_some());
        // Closing again is rejected.
        assert!(p.close().is_err());
    }
}
