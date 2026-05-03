-- Demand planning: reorder_policies
--
-- Per-(variant, store) configuration for the replenishment generator. The
-- `version` column drives optimistic locking on update.
--
-- See modules/demand_planning.

CREATE TABLE IF NOT EXISTS reorder_policies (
    id                   UUID PRIMARY KEY,
    product_variant_id   UUID NOT NULL,
    store_id             UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    min_qty              NUMERIC(20, 4) NOT NULL,
    max_qty              NUMERIC(20, 4) NOT NULL,
    lead_time_days       INTEGER NOT NULL,
    safety_stock_qty     NUMERIC(20, 4) NOT NULL DEFAULT 0,
    review_cycle_days    INTEGER NOT NULL DEFAULT 7,
    preferred_vendor_id  UUID NULL REFERENCES vendors(id) ON DELETE SET NULL,
    is_active            BOOLEAN NOT NULL DEFAULT TRUE,
    version              INTEGER NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT reorder_policies_qty_nonneg
        CHECK (min_qty >= 0 AND max_qty >= 0 AND safety_stock_qty >= 0),
    CONSTRAINT reorder_policies_qty_range CHECK (max_qty >= min_qty),
    CONSTRAINT reorder_policies_days_positive
        CHECK (lead_time_days > 0 AND review_cycle_days > 0),
    CONSTRAINT reorder_policies_variant_store_unique
        UNIQUE (product_variant_id, store_id)
);

CREATE INDEX IF NOT EXISTS idx_reorder_policies_store
    ON reorder_policies (store_id) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_reorder_policies_active
    ON reorder_policies (is_active) WHERE is_active = TRUE;
