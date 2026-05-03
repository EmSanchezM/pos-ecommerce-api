-- Restaurant operations: dining tables.
--
-- v1.0 has no FloorPlan with coordinates yet — v1.2 adds `floor_plan_id` and
-- a layout JSON. `current_ticket_id` is intentionally a free UUID (no FK to
-- kds_tickets) so a table can survive a ticket being deleted without a
-- cascade dance.

CREATE TABLE IF NOT EXISTS restaurant_tables (
    id                UUID PRIMARY KEY,
    store_id          UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    label             VARCHAR(80) NOT NULL,
    capacity          INTEGER NOT NULL,
    status            VARCHAR(16) NOT NULL DEFAULT 'free',
    current_ticket_id UUID NULL,
    notes             TEXT NULL,
    is_active         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT restaurant_tables_capacity_chk CHECK (capacity > 0),
    CONSTRAINT restaurant_tables_status_chk
        CHECK (status IN ('free', 'seated', 'reserved', 'dirty'))
);

CREATE INDEX IF NOT EXISTS idx_restaurant_tables_store_active
    ON restaurant_tables (store_id, label)
    WHERE is_active = TRUE;
