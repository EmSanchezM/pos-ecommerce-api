-- Demand planning: abc_classifications
--
-- Pareto-style classification of variants by revenue contribution within a
-- window. Recomputed monthly; one row per (variant, store, period_start,
-- period_end).
--
-- See modules/demand_planning.

CREATE TABLE IF NOT EXISTS abc_classifications (
    id                   UUID PRIMARY KEY,
    product_variant_id   UUID NOT NULL,
    store_id             UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    period_start         DATE NOT NULL,
    period_end           DATE NOT NULL,
    revenue_share        NUMERIC(7, 6) NOT NULL,
    abc_class            CHAR(1) NOT NULL,
    classified_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT abc_classifications_class_check CHECK (abc_class IN ('A', 'B', 'C')),
    CONSTRAINT abc_classifications_share_check
        CHECK (revenue_share >= 0 AND revenue_share <= 1),
    CONSTRAINT abc_classifications_range_check CHECK (period_start <= period_end),
    CONSTRAINT abc_classifications_unique
        UNIQUE (product_variant_id, store_id, period_start, period_end)
);

CREATE INDEX IF NOT EXISTS idx_abc_classifications_store_class
    ON abc_classifications (store_id, abc_class, revenue_share DESC);
