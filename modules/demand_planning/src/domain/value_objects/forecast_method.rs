//! ForecastMethod — which statistical algorithm produced a forecast.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::DemandPlanningError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastMethod {
    MovingAverage3,
    MovingAverage6,
    ExponentialSmoothing,
    HoltWinters,
    Manual,
}

impl fmt::Display for ForecastMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ForecastMethod::MovingAverage3 => "moving_average_3",
            ForecastMethod::MovingAverage6 => "moving_average_6",
            ForecastMethod::ExponentialSmoothing => "exponential_smoothing",
            ForecastMethod::HoltWinters => "holt_winters",
            ForecastMethod::Manual => "manual",
        };
        f.write_str(s)
    }
}

impl FromStr for ForecastMethod {
    type Err = DemandPlanningError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moving_average_3" => Ok(Self::MovingAverage3),
            "moving_average_6" => Ok(Self::MovingAverage6),
            "exponential_smoothing" => Ok(Self::ExponentialSmoothing),
            "holt_winters" => Ok(Self::HoltWinters),
            "manual" => Ok(Self::Manual),
            other => Err(DemandPlanningError::InvalidForecastMethod(other.into())),
        }
    }
}
