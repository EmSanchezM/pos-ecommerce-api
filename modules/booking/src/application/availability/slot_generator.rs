//! Pure slot grid generation. No DB, no Tokio — easily proptest-able.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};

use crate::domain::value_objects::TimeSlot;

/// A working window inside a single day, expressed as wall-clock NaiveTimes
/// that the calendar repository returned.
#[derive(Debug, Clone, Copy)]
pub struct WorkingWindow {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

/// Generates candidate time slots for one calendar day.
///
/// - `date` is the day to generate for (UTC).
/// - `windows` are working windows for the day (often a single 9-17 block, but
///   can be multiple e.g. for split shifts).
/// - `service_duration_minutes` is the chargeable duration; the slot end is
///   `starts_at + duration`.
/// - `buffer_before` / `buffer_after` extend the *blocked* time around the
///   slot but do not appear in the returned `TimeSlot.ends_at` (the slot
///   exposed to the customer is the chargeable window only).
/// - `granularity_minutes` is the grid step (e.g. 15 yields 09:00, 09:15, …).
pub fn generate_slots(
    date: NaiveDate,
    windows: &[WorkingWindow],
    service_duration_minutes: u32,
    buffer_before: u32,
    buffer_after: u32,
    granularity_minutes: u32,
) -> Vec<TimeSlot> {
    if service_duration_minutes == 0 || granularity_minutes == 0 || windows.is_empty() {
        return Vec::new();
    }
    let duration = Duration::minutes(service_duration_minutes as i64);
    let buffer_b = Duration::minutes(buffer_before as i64);
    let buffer_a = Duration::minutes(buffer_after as i64);
    let grid_step = Duration::minutes(granularity_minutes as i64);

    let mut out = Vec::new();
    for w in windows {
        if w.end <= w.start {
            continue;
        }
        let window_start: DateTime<Utc> = Utc.from_utc_datetime(&date.and_time(w.start));
        let window_end: DateTime<Utc> = Utc.from_utc_datetime(&date.and_time(w.end));

        // The earliest a slot can start is window_start + buffer_before; the
        // latest it can end is window_end - buffer_after.
        let earliest_start = window_start + buffer_b;
        let latest_end = window_end - buffer_a;
        if latest_end <= earliest_start {
            continue;
        }

        let mut cursor = align_up(earliest_start, grid_step);
        while cursor + duration <= latest_end {
            out.push(TimeSlot {
                starts_at: cursor,
                ends_at: cursor + duration,
            });
            cursor += grid_step;
        }
    }
    out
}

/// Round a `DateTime<Utc>` up to the nearest multiple of `step` seconds since
/// the Unix epoch. Keeps the cursor on a clean grid even when the working
/// window starts on a fractional minute.
fn align_up(t: DateTime<Utc>, step: Duration) -> DateTime<Utc> {
    let step_secs = step.num_seconds();
    if step_secs <= 0 {
        return t;
    }
    let secs = t.timestamp();
    let remainder = secs.rem_euclid(step_secs);
    if remainder == 0 {
        t
    } else {
        t + Duration::seconds(step_secs - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }
    fn t(s: &str) -> NaiveTime {
        NaiveTime::parse_from_str(s, "%H:%M").unwrap()
    }

    #[test]
    fn nine_to_five_thirty_min_fifteen_grid() {
        let slots = generate_slots(
            d("2026-05-04"),
            &[WorkingWindow {
                start: t("09:00"),
                end: t("17:00"),
            }],
            30,
            0,
            0,
            15,
        );
        // 8h window, 30min slot, 15min step → 31 candidates
        // (09:00..16:30 inclusive with 15-min steps)
        assert_eq!(slots.len(), 31);
        assert_eq!(
            slots.first().unwrap().starts_at.to_rfc3339(),
            "2026-05-04T09:00:00+00:00"
        );
        assert_eq!(
            slots.last().unwrap().starts_at.to_rfc3339(),
            "2026-05-04T16:30:00+00:00"
        );
    }

    #[test]
    fn buffers_shrink_the_usable_window() {
        // 30min slot in 9-10 with 10min before+after → window is effectively 9:10..9:50 → 0 slots
        let slots = generate_slots(
            d("2026-05-04"),
            &[WorkingWindow {
                start: t("09:00"),
                end: t("10:00"),
            }],
            30,
            10,
            10,
            15,
        );
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].starts_at.to_rfc3339(), "2026-05-04T09:15:00+00:00");
    }

    #[test]
    fn empty_windows_yield_nothing() {
        let slots = generate_slots(d("2026-05-04"), &[], 30, 0, 0, 15);
        assert!(slots.is_empty());
    }

    #[test]
    fn inverted_window_yields_nothing() {
        let slots = generate_slots(
            d("2026-05-04"),
            &[WorkingWindow {
                start: t("17:00"),
                end: t("09:00"),
            }],
            30,
            0,
            0,
            15,
        );
        assert!(slots.is_empty());
    }
}
