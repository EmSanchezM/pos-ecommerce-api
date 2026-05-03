-- Booking: resources (people, equipment, rooms) that can perform services.
--
-- A `Resource` is store-scoped. v1.0 supports three types:
--   - person     : stylist, mechanic, instructor
--   - equipment  : chair, lift, lane
--   - room       : treatment room, conference room
--
-- See modules/booking.

CREATE TABLE IF NOT EXISTS booking_resources (
    id            UUID PRIMARY KEY,
    store_id      UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    resource_type VARCHAR(16) NOT NULL,
    name          VARCHAR(120) NOT NULL,
    color         VARCHAR(7) NULL,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT booking_resources_type_chk
        CHECK (resource_type IN ('person', 'equipment', 'room'))
);

CREATE INDEX IF NOT EXISTS idx_booking_resources_store_active
    ON booking_resources (store_id)
    WHERE is_active = TRUE;
