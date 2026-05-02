-- Demand planning: demand_forecasts
--
-- One row per (variant, store, period_start, method) tuple. Older rows are
-- replaced via ON CONFLICT in the recompute job; the table is append-with-
-- upsert and pruned by `delete_older_than`.
--
-- See modules/demand_planning.

CREATE TABLE IF NOT EXISTS demand_forecasts (
    id                  UUID PRIMARY KEY,
    -- Both products (no variants) and variants live in the same column;
    -- callers use COALESCE(variant_id, product_id) when referencing.
    product_variant_id  UUID NOT NULL,
    store_id            UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    period              VARCHAR(16) NOT NULL,
    period_start        DATE NOT NULL,
    period_end          DATE NOT NULL,
    method              VARCHAR(32) NOT NULL,
    forecasted_qty      NUMERIC(20, 4) NOT NULL,
    confidence_low      NUMERIC(20, 4) NOT NULL,
    confidence_high     NUMERIC(20, 4) NOT NULL,
    computed_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT demand_forecasts_period_check
        CHECK (period IN ('daily', 'weekly', 'monthly')),
    CONSTRAINT demand_forecasts_range_check CHECK (period_start <= period_end),
    CONSTRAINT demand_forecasts_unique
        UNIQUE (product_variant_id, store_id, period, period_start, method)
);

CREATE INDEX IF NOT EXISTS idx_demand_forecasts_lookup
    ON demand_forecasts (product_variant_id, store_id, computed_at DESC);

CREATE INDEX IF NOT EXISTS idx_demand_forecasts_store
    ON demand_forecasts (store_id, computed_at DESC);
