-- Service order line items.
--
-- Two types: 'labor' (technician time, no inventory link) and 'part' (a
-- product whose `product_id` may be a reference to an inventory product).
-- `total = quantity * unit_price + tax_amount`. `tax_rate` is captured for
-- audit; the application layer computes `tax_amount` from it at create time.

CREATE TABLE IF NOT EXISTS service_order_items (
    id               UUID PRIMARY KEY,
    service_order_id UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    item_type        VARCHAR(8)  NOT NULL,
    description      TEXT        NOT NULL,
    quantity         NUMERIC(20, 4) NOT NULL,
    unit_price       NUMERIC(20, 4) NOT NULL,
    total            NUMERIC(20, 4) NOT NULL,
    product_id       UUID        NULL,
    variant_id       UUID        NULL,
    tax_rate         NUMERIC(5, 4) NOT NULL DEFAULT 0,
    tax_amount       NUMERIC(20, 4) NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT service_order_items_type_chk
        CHECK (item_type IN ('labor', 'part')),
    CONSTRAINT service_order_items_qty_chk     CHECK (quantity   > 0),
    CONSTRAINT service_order_items_price_chk   CHECK (unit_price >= 0),
    CONSTRAINT service_order_items_tax_chk     CHECK (tax_rate   >= 0)
);

CREATE INDEX IF NOT EXISTS idx_service_order_items_order
    ON service_order_items (service_order_id);
