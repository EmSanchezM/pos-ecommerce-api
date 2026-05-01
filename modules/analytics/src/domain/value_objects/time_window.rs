//! TimeWindow — canonical periods used to scope KPI snapshots and reports.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::AnalyticsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeWindow {
    Today,
    ThisWeek,
    ThisMonth,
    ThisYear,
    Last7Days,
    Last30Days,
    AllTime,
}

impl TimeWindow {
    /// Resolve the window into an absolute `[from, to)` range, evaluated at `now`.
    pub fn bounds(self, now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        let to = now;
        let from = match self {
            TimeWindow::Today => start_of_day(now),
            TimeWindow::Last7Days => now - Duration::days(7),
            TimeWindow::Last30Days => now - Duration::days(30),
            TimeWindow::ThisWeek => start_of_week(now),
            TimeWindow::ThisMonth => start_of_month(now),
            TimeWindow::ThisYear => start_of_year(now),
            TimeWindow::AllTime => Utc.timestamp_opt(0, 0).single().unwrap_or(now),
        };
        (from, to)
    }
}

impl fmt::Display for TimeWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeWindow::Today => "today",
            TimeWindow::ThisWeek => "this_week",
            TimeWindow::ThisMonth => "this_month",
            TimeWindow::ThisYear => "this_year",
            TimeWindow::Last7Days => "last_7_days",
            TimeWindow::Last30Days => "last_30_days",
            TimeWindow::AllTime => "all_time",
        };
        f.write_str(s)
    }
}

impl FromStr for TimeWindow {
    type Err = AnalyticsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "today" => Ok(Self::Today),
            "this_week" => Ok(Self::ThisWeek),
            "this_month" => Ok(Self::ThisMonth),
            "this_year" => Ok(Self::ThisYear),
            "last_7_days" => Ok(Self::Last7Days),
            "last_30_days" => Ok(Self::Last30Days),
            "all_time" => Ok(Self::AllTime),
            other => Err(AnalyticsError::InvalidTimeWindow(other.into())),
        }
    }
}

fn start_of_day(t: DateTime<Utc>) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(t.year(), t.month(), t.day(), 0, 0, 0)
        .single()
        .unwrap_or(t)
}

fn start_of_week(t: DateTime<Utc>) -> DateTime<Utc> {
    // ISO weeks start on Monday. weekday().num_days_from_monday() == 0 on Monday.
    let days_back = t.weekday().num_days_from_monday() as i64;
    start_of_day(t) - Duration::days(days_back)
}

fn start_of_month(t: DateTime<Utc>) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(t.year(), t.month(), 1, 0, 0, 0)
        .single()
        .unwrap_or(t)
}

fn start_of_year(t: DateTime<Utc>) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(t.year(), 1, 1, 0, 0, 0)
        .single()
        .unwrap_or(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn parses_known_windows() {
        assert_eq!("today".parse::<TimeWindow>().unwrap(), TimeWindow::Today);
        assert_eq!(
            "last_30_days".parse::<TimeWindow>().unwrap(),
            TimeWindow::Last30Days
        );
        assert!("nope".parse::<TimeWindow>().is_err());
    }

    #[test]
    fn today_bounds_start_at_midnight() {
        let now = Utc.with_ymd_and_hms(2026, 5, 15, 13, 30, 0).unwrap();
        let (from, to) = TimeWindow::Today.bounds(now);
        assert_eq!(from.hour(), 0);
        assert_eq!(from.minute(), 0);
        assert_eq!(to, now);
    }

    #[test]
    fn last_7_days_subtracts_a_week() {
        let now = Utc.with_ymd_and_hms(2026, 5, 15, 13, 30, 0).unwrap();
        let (from, to) = TimeWindow::Last7Days.bounds(now);
        assert_eq!(to - from, Duration::days(7));
    }
}
