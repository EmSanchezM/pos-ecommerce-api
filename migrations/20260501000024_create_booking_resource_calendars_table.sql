-- Booking: recurring weekly availability windows per resource.
--
-- One row per (resource, day_of_week, start_time). Multiple rows on the same
-- day model split shifts (e.g. 09-13, 15-19). Exceptions (vacations, holidays)
-- ship in v1.2 as a sibling table.

CREATE TABLE IF NOT EXISTS booking_resource_calendars (
    id          UUID PRIMARY KEY,
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE CASCADE,
    day_of_week SMALLINT NOT NULL,
    start_time  TIME NOT NULL,
    end_time    TIME NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,

    CONSTRAINT booking_resource_calendars_dow_chk
        CHECK (day_of_week BETWEEN 0 AND 6),
    CONSTRAINT booking_resource_calendars_window_chk
        CHECK (end_time > start_time)
);

CREATE INDEX IF NOT EXISTS idx_booking_resource_calendars_lookup
    ON booking_resource_calendars (resource_id, day_of_week)
    WHERE is_active = TRUE;
