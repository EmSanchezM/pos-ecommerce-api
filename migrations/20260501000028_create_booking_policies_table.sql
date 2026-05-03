-- Booking: per-store policy.
--
-- v1.0 only enforces `cancellation_window_hours`. The deposit and no-show fee
-- columns are captured for v1.1 when the payments gateway integration ships.

CREATE TABLE IF NOT EXISTS booking_policies (
    id                       UUID PRIMARY KEY,
    store_id                 UUID NOT NULL UNIQUE REFERENCES stores(id) ON DELETE CASCADE,
    requires_deposit         BOOLEAN NOT NULL DEFAULT FALSE,
    deposit_percentage       NUMERIC(5, 2) NULL,
    cancellation_window_hours INTEGER NOT NULL DEFAULT 24,
    no_show_fee_amount       NUMERIC(20, 4) NULL,
    default_buffer_minutes   INTEGER NOT NULL DEFAULT 0,
    advance_booking_days_max INTEGER NOT NULL DEFAULT 60,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT booking_policies_window_chk    CHECK (cancellation_window_hours >= 0),
    CONSTRAINT booking_policies_buffer_chk    CHECK (default_buffer_minutes    >= 0),
    CONSTRAINT booking_policies_advance_chk   CHECK (advance_booking_days_max  >  0)
);
