-- Booking: appointments (the booking aggregate root).
--
-- `customer_id` is nullable so public/unauthenticated bookings can store the
-- contact via the snapshot fields without forcing a registered Customer.
-- `public_token` is an unguessable per-appointment token used by the public
-- "view your booking" endpoint so visitors can check status without auth.
--
-- The partial index on (resource_id, starts_at, ends_at) WHERE status IN
-- (...) is what `save_with_slot_check` scans under FOR UPDATE to detect
-- double-booking under concurrency. Terminal states do not occupy the slot.

CREATE TABLE IF NOT EXISTS booking_appointments (
    id                      UUID PRIMARY KEY,
    store_id                UUID NOT NULL REFERENCES stores(id)             ON DELETE CASCADE,
    service_id              UUID NOT NULL REFERENCES booking_services(id)   ON DELETE RESTRICT,
    resource_id             UUID NOT NULL REFERENCES booking_resources(id)  ON DELETE RESTRICT,
    customer_id             UUID NULL     REFERENCES customers(id)          ON DELETE SET NULL,
    customer_name           VARCHAR(120)  NOT NULL,
    customer_email          VARCHAR(160)  NOT NULL,
    customer_phone          VARCHAR(40)   NULL,
    starts_at               TIMESTAMPTZ   NOT NULL,
    ends_at                 TIMESTAMPTZ   NOT NULL,
    status                  VARCHAR(16)   NOT NULL DEFAULT 'scheduled',
    deposit_transaction_id  UUID          NULL,
    generated_sale_id       UUID          NULL,
    notes                   TEXT          NULL,
    canceled_reason         TEXT          NULL,
    no_show_at              TIMESTAMPTZ   NULL,
    public_token            VARCHAR(64)   NOT NULL,
    created_by              UUID          NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT booking_appointments_status_chk
        CHECK (status IN ('scheduled', 'confirmed', 'in_progress',
                          'completed', 'canceled', 'no_show')),
    CONSTRAINT booking_appointments_window_chk
        CHECK (ends_at > starts_at)
);

CREATE INDEX IF NOT EXISTS idx_booking_appointments_resource_window
    ON booking_appointments (resource_id, starts_at, ends_at)
    WHERE status IN ('scheduled', 'confirmed', 'in_progress');

CREATE INDEX IF NOT EXISTS idx_booking_appointments_store
    ON booking_appointments (store_id, starts_at DESC);

CREATE INDEX IF NOT EXISTS idx_booking_appointments_customer
    ON booking_appointments (customer_id)
    WHERE customer_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_booking_appointments_token
    ON booking_appointments (public_token);
