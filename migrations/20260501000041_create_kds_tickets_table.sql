-- Restaurant operations: KDS tickets — the kitchen display aggregate root.
--
-- Workflow: pending → in_progress → ready → served. cancel from any
-- non-terminal state. `ticket_number` is a per-store sequence (the
-- application picks MAX(ticket_number)+1 under FOR UPDATE on existing rows).
--
-- `sale_id` is nullable: v1.0 lets the staff create tickets directly from
-- the KDS endpoints without a sales::Sale; v1.1 wires the auto-generation
-- from `sale.item_added` events and populates the link.

CREATE TABLE IF NOT EXISTS kds_tickets (
    id              UUID PRIMARY KEY,
    store_id        UUID NOT NULL REFERENCES stores(id)            ON DELETE CASCADE,
    station_id      UUID NOT NULL REFERENCES kitchen_stations(id)  ON DELETE RESTRICT,
    table_id        UUID NULL     REFERENCES restaurant_tables(id) ON DELETE SET NULL,
    sale_id         UUID NULL,
    ticket_number   INTEGER NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',
    course          VARCHAR(16) NOT NULL DEFAULT 'main',
    notes           TEXT NULL,
    sent_at         TIMESTAMPTZ NULL,
    ready_at        TIMESTAMPTZ NULL,
    served_at       TIMESTAMPTZ NULL,
    canceled_reason TEXT NULL,
    created_by      UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT kds_tickets_status_chk
        CHECK (status IN ('pending', 'in_progress', 'ready', 'served', 'canceled')),
    CONSTRAINT kds_tickets_course_chk
        CHECK (course IN ('appetizer', 'main', 'dessert', 'drink', 'other')),
    CONSTRAINT kds_tickets_number_chk    CHECK (ticket_number > 0),
    CONSTRAINT kds_tickets_unique_number UNIQUE (store_id, ticket_number)
);

CREATE INDEX IF NOT EXISTS idx_kds_tickets_station_active
    ON kds_tickets (station_id, status, created_at)
    WHERE status IN ('pending', 'in_progress', 'ready');

CREATE INDEX IF NOT EXISTS idx_kds_tickets_table
    ON kds_tickets (table_id)
    WHERE table_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_kds_tickets_store_recent
    ON kds_tickets (store_id, created_at DESC);
