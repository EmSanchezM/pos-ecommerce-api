//! Recurring weekly availability for a resource. One row per (resource, day).
//! v1.0 does not model exceptions (vacations / holidays); that ships in v1.2 as
//! a sibling table `booking_resource_calendar_exceptions`.

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::BookingError;
use crate::domain::value_objects::{ResourceCalendarId, ResourceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCalendar {
    id: ResourceCalendarId,
    resource_id: ResourceId,
    /// 0 = Sunday, 6 = Saturday (matches chrono::Weekday::num_days_from_sunday).
    day_of_week: i16,
    start_time: NaiveTime,
    end_time: NaiveTime,
    is_active: bool,
}

impl ResourceCalendar {
    pub fn new(
        resource_id: ResourceId,
        day_of_week: i16,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> Result<Self, BookingError> {
        if !(0..=6).contains(&day_of_week) {
            return Err(BookingError::Validation(format!(
                "day_of_week must be 0..=6 (got {})",
                day_of_week
            )));
        }
        if end_time <= start_time {
            return Err(BookingError::InvalidTimeRange);
        }
        Ok(Self {
            id: ResourceCalendarId::new(),
            resource_id,
            day_of_week,
            start_time,
            end_time,
            is_active: true,
        })
    }

    pub fn reconstitute(
        id: ResourceCalendarId,
        resource_id: ResourceId,
        day_of_week: i16,
        start_time: NaiveTime,
        end_time: NaiveTime,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            resource_id,
            day_of_week,
            start_time,
            end_time,
            is_active,
        }
    }

    pub fn id(&self) -> ResourceCalendarId {
        self.id
    }
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
    pub fn day_of_week(&self) -> i16 {
        self.day_of_week
    }
    pub fn start_time(&self) -> NaiveTime {
        self.start_time
    }
    pub fn end_time(&self) -> NaiveTime {
        self.end_time
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
