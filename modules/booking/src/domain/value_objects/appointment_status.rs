use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::BookingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    InProgress,
    Completed,
    Canceled,
    NoShow,
}

impl AppointmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AppointmentStatus::Scheduled => "scheduled",
            AppointmentStatus::Confirmed => "confirmed",
            AppointmentStatus::InProgress => "in_progress",
            AppointmentStatus::Completed => "completed",
            AppointmentStatus::Canceled => "canceled",
            AppointmentStatus::NoShow => "no_show",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            AppointmentStatus::Completed | AppointmentStatus::Canceled | AppointmentStatus::NoShow
        )
    }

    pub fn occupies_slot(self) -> bool {
        matches!(
            self,
            AppointmentStatus::Scheduled
                | AppointmentStatus::Confirmed
                | AppointmentStatus::InProgress
        )
    }

    pub fn can_transition_to(self, other: AppointmentStatus) -> bool {
        use AppointmentStatus::*;
        matches!(
            (self, other),
            (Scheduled, Confirmed)
                | (Scheduled, Canceled)
                | (Confirmed, InProgress)
                | (Confirmed, Canceled)
                | (Confirmed, NoShow)
                | (InProgress, Completed)
                | (InProgress, NoShow)
        )
    }
}

impl FromStr for AppointmentStatus {
    type Err = BookingError;
    fn from_str(s: &str) -> Result<Self, BookingError> {
        match s {
            "scheduled" => Ok(AppointmentStatus::Scheduled),
            "confirmed" => Ok(AppointmentStatus::Confirmed),
            "in_progress" => Ok(AppointmentStatus::InProgress),
            "completed" => Ok(AppointmentStatus::Completed),
            "canceled" => Ok(AppointmentStatus::Canceled),
            "no_show" => Ok(AppointmentStatus::NoShow),
            other => Err(BookingError::InvalidAppointmentStatus(other.to_string())),
        }
    }
}
