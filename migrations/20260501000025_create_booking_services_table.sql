-- Booking: bookable services (haircut, oil change, massage).
--
-- `duration_minutes` is the chargeable window; `buffer_minutes_before/after`
-- block extra time around the slot (cleanup, prep) without billing for it.
-- Services are linked to one or more eligible resources via the M2M
-- `booking_service_resources` table.

CREATE TABLE IF NOT EXISTS booking_services (
    id                    UUID PRIMARY KEY,
    store_id              UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    name                  VARCHAR(120) NOT NULL,
    description           TEXT NULL,
    duration_minutes      INTEGER NOT NULL,
    price                 NUMERIC(20, 4) NOT NULL DEFAULT 0,
    buffer_minutes_before INTEGER NOT NULL DEFAULT 0,
    buffer_minutes_after  INTEGER NOT NULL DEFAULT 0,
    requires_deposit      BOOLEAN NOT NULL DEFAULT FALSE,
    deposit_amount        NUMERIC(20, 4) NULL,
    is_active             BOOLEAN NOT NULL DEFAULT TRUE,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT booking_services_duration_chk     CHECK (duration_minutes > 0),
    CONSTRAINT booking_services_buf_before_chk   CHECK (buffer_minutes_before >= 0),
    CONSTRAINT booking_services_buf_after_chk    CHECK (buffer_minutes_after  >= 0),
    CONSTRAINT booking_services_deposit_chk      CHECK (
        NOT requires_deposit OR deposit_amount IS NOT NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_booking_services_store_active
    ON booking_services (store_id)
    WHERE is_active = TRUE;
