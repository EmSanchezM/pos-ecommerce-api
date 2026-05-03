-- Demand planning: replenishment_suggestions
--
-- Pending → approved → ordered, or pending → dismissed. Once approved the
-- module creates a draft Purchase Order via the purchasing module and stores
-- its id in `generated_purchase_order_id`.
--
-- See modules/demand_planning.

CREATE TABLE IF NOT EXISTS replenishment_suggestions (
    id                            UUID PRIMARY KEY,
    product_variant_id            UUID NOT NULL,
    store_id                      UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    current_stock                 NUMERIC(20, 4) NOT NULL,
    forecast_qty                  NUMERIC(20, 4) NOT NULL,
    recommended_qty               NUMERIC(20, 4) NOT NULL,
    suggested_vendor_id           UUID NULL REFERENCES vendors(id) ON DELETE SET NULL,
    status                        VARCHAR(16) NOT NULL DEFAULT 'pending',
    generated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_at                    TIMESTAMPTZ NULL,
    decided_by                    UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    generated_purchase_order_id   UUID NULL REFERENCES purchase_orders(id) ON DELETE SET NULL,
    dismiss_reason                TEXT NULL,

    CONSTRAINT replenishment_suggestions_status_check
        CHECK (status IN ('pending', 'approved', 'ordered', 'dismissed')),
    CONSTRAINT replenishment_suggestions_qty_nonneg
        CHECK (current_stock >= 0 AND recommended_qty >= 0)
);

-- Hot-path index for the operator's "what should I order?" view.
CREATE INDEX IF NOT EXISTS idx_replenishment_suggestions_pending
    ON replenishment_suggestions (store_id, status)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_replenishment_suggestions_variant_store
    ON replenishment_suggestions (product_variant_id, store_id, generated_at DESC);
