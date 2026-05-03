-- Service orders: workshop ticket aggregate root.
--
-- Workflow (9 statuses; terminal = delivered, canceled):
--   intake → diagnosis → quote_sent → quote_approved → in_repair → testing
--   → ready_for_pickup → delivered
-- + cancel from any non-terminal state.
--
-- `customer_id` is nullable for walk-ins (snapshot fields cover the contact);
-- `public_token` is an unguessable per-order token used by the public status
-- endpoint so customers can check progress without auth (mirrors booking).
-- `total_amount` is a denormalised cache: the application layer recalculates
-- via SUM(service_order_items.total) on every item mutation.

CREATE TABLE IF NOT EXISTS service_orders (
    id                 UUID PRIMARY KEY,
    store_id           UUID NOT NULL REFERENCES stores(id)         ON DELETE CASCADE,
    asset_id           UUID NOT NULL REFERENCES service_assets(id) ON DELETE RESTRICT,
    customer_id        UUID NULL     REFERENCES customers(id)      ON DELETE SET NULL,
    customer_name      VARCHAR(120) NOT NULL,
    customer_email     VARCHAR(160) NOT NULL,
    customer_phone     VARCHAR(40)  NULL,
    status             VARCHAR(20)  NOT NULL DEFAULT 'intake',
    priority           VARCHAR(10)  NOT NULL DEFAULT 'normal',
    intake_notes       TEXT         NULL,
    intake_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    intake_by_user_id  UUID         NULL REFERENCES users(id) ON DELETE SET NULL,
    promised_at        TIMESTAMPTZ  NULL,
    delivered_at       TIMESTAMPTZ  NULL,
    generated_sale_id  UUID         NULL,
    canceled_reason    TEXT         NULL,
    canceled_at        TIMESTAMPTZ  NULL,
    public_token       VARCHAR(64)  NOT NULL,
    total_amount       NUMERIC(20, 4) NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT service_orders_status_chk
        CHECK (status IN ('intake', 'diagnosis', 'quote_sent', 'quote_approved',
                          'in_repair', 'testing', 'ready_for_pickup',
                          'delivered', 'canceled')),
    CONSTRAINT service_orders_priority_chk
        CHECK (priority IN ('low', 'normal', 'high', 'urgent'))
);

CREATE INDEX IF NOT EXISTS idx_service_orders_store_status
    ON service_orders (store_id, status);

CREATE INDEX IF NOT EXISTS idx_service_orders_asset
    ON service_orders (asset_id, intake_at DESC);

CREATE INDEX IF NOT EXISTS idx_service_orders_customer
    ON service_orders (customer_id)
    WHERE customer_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_service_orders_token
    ON service_orders (public_token);
