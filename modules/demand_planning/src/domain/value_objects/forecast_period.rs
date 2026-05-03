//! ForecastPeriod — the granularity of a forecast horizon.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::DemandPlanningError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl ForecastPeriod {
    /// Approximate number of days covered by one period (used for safety stock
    /// scaling against a daily lead time).
    pub fn days(self) -> i32 {
        match self {
            ForecastPeriod::Daily => 1,
            ForecastPeriod::Weekly => 7,
            ForecastPeriod::Monthly => 30,
        }
    }
}

impl fmt::Display for ForecastPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ForecastPeriod::Daily => "daily",
            ForecastPeriod::Weekly => "weekly",
            ForecastPeriod::Monthly => "monthly",
        };
        f.write_str(s)
    }
}

impl FromStr for ForecastPeriod {
    type Err = DemandPlanningError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            other => Err(DemandPlanningError::InvalidForecastPeriod(other.into())),
        }
    }
}
