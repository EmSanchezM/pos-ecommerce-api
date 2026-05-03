//! Pure slot subtraction: given candidate slots and booked time ranges, drop
//! candidates that overlap any booked range.

use crate::domain::value_objects::TimeSlot;

pub fn subtract_booked(candidates: Vec<TimeSlot>, booked: &[TimeSlot]) -> Vec<TimeSlot> {
    candidates
        .into_iter()
        .filter(|c| !booked.iter().any(|b| c.overlaps(b)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn slot(start: &str, end: &str) -> TimeSlot {
        TimeSlot {
            starts_at: NaiveDateTime::parse_from_str(start, "%Y-%m-%dT%H:%M:%S")
                .unwrap()
                .and_utc(),
            ends_at: NaiveDateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%S")
                .unwrap()
                .and_utc(),
        }
    }

    #[test]
    fn drops_overlapping_candidate() {
        let candidates = vec![
            slot("2026-05-04T09:00:00", "2026-05-04T09:30:00"),
            slot("2026-05-04T10:00:00", "2026-05-04T10:30:00"),
        ];
        let booked = vec![slot("2026-05-04T09:00:00", "2026-05-04T09:30:00")];
        let out = subtract_booked(candidates, &booked);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].starts_at.to_rfc3339(), "2026-05-04T10:00:00+00:00");
    }

    #[test]
    fn touching_slots_do_not_overlap() {
        // Booking ends at 09:30, next candidate starts at 09:30 — they touch
        // but should not be considered overlapping.
        let candidates = vec![slot("2026-05-04T09:30:00", "2026-05-04T10:00:00")];
        let booked = vec![slot("2026-05-04T09:00:00", "2026-05-04T09:30:00")];
        let out = subtract_booked(candidates, &booked);
        assert_eq!(out.len(), 1);
    }
}
