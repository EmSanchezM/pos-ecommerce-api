//! JournalEntryStatus — workflow states for a journal entry.
//!
//! draft → posted → voided. `posted` is the only status that contributes to
//! reports. `voided` requires writing a reversing entry rather than mutating
//! the original — voiding here is metadata only.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AccountingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalEntryStatus {
    Draft,
    Posted,
    Voided,
}

impl fmt::Display for JournalEntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            JournalEntryStatus::Draft => "draft",
            JournalEntryStatus::Posted => "posted",
            JournalEntryStatus::Voided => "voided",
        };
        f.write_str(s)
    }
}

impl FromStr for JournalEntryStatus {
    type Err = AccountingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "posted" => Ok(Self::Posted),
            "voided" => Ok(Self::Voided),
            other => Err(AccountingError::InvalidJournalEntryStatus(other.into())),
        }
    }
}
