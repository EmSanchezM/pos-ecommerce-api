use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::BookingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSlot {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

impl TimeSlot {
    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<Self, BookingError> {
        if ends_at <= starts_at {
            return Err(BookingError::InvalidTimeRange);
        }
        Ok(Self { starts_at, ends_at })
    }

    pub fn from_start_duration(starts_at: DateTime<Utc>, duration_minutes: i64) -> Self {
        Self {
            starts_at,
            ends_at: starts_at + Duration::minutes(duration_minutes),
        }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.ends_at - self.starts_at).num_minutes()
    }

    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.starts_at < other.ends_at && other.starts_at < self.ends_at
    }
}
