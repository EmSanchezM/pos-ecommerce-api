-- Restaurant operations: KDS ticket items.
--
-- `modifiers_summary` is a denormalised text the application builds from the
-- selected modifier ids ("sin cebolla, extra queso, término medio"). v1.2
-- can add a sibling table `kds_ticket_item_modifiers` if granular reporting
-- on modifier usage becomes useful.
--
-- Item workflow mirrors the parent ticket's: pending → in_progress → ready
-- → served. Cancellation is allowed from pending / in_progress.

CREATE TABLE IF NOT EXISTS kds_ticket_items (
    id                   UUID PRIMARY KEY,
    ticket_id            UUID NOT NULL REFERENCES kds_tickets(id) ON DELETE CASCADE,
    sale_item_id         UUID NULL,
    product_id           UUID NULL,
    description          TEXT NOT NULL,
    quantity             NUMERIC(20, 4) NOT NULL,
    modifiers_summary    TEXT NOT NULL DEFAULT '',
    special_instructions TEXT NULL,
    status               VARCHAR(16) NOT NULL DEFAULT 'pending',
    ready_at             TIMESTAMPTZ NULL,
    served_at            TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT kds_ticket_items_qty_chk    CHECK (quantity > 0),
    CONSTRAINT kds_ticket_items_status_chk
        CHECK (status IN ('pending', 'in_progress', 'ready', 'served', 'canceled'))
);

CREATE INDEX IF NOT EXISTS idx_kds_ticket_items_ticket
    ON kds_ticket_items (ticket_id, created_at);
