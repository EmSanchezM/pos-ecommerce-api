-- Loyalty: member_tiers
--
-- Bronze/Silver/Gold per program with cumulative-earn thresholds. `benefits`
-- is opaque JSONB consumed by the storefront — this module doesn't enforce
-- discounts/perks, only tier classification.
--
-- See modules/loyalty.

CREATE TABLE IF NOT EXISTS loyalty_member_tiers (
    id               UUID PRIMARY KEY,
    program_id       UUID NOT NULL REFERENCES loyalty_programs(id) ON DELETE CASCADE,
    name             VARCHAR(64) NOT NULL,
    threshold_points BIGINT NOT NULL,
    benefits         JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    is_active        BOOLEAN NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT loyalty_member_tiers_threshold_nonneg CHECK (threshold_points >= 0),
    CONSTRAINT loyalty_member_tiers_program_name_unique UNIQUE (program_id, name)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_member_tiers_program
    ON loyalty_member_tiers (program_id, threshold_points);
