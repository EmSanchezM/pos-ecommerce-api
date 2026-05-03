-- Loyalty: programs
--
-- One row per (store, name) loyalty program. The conversion rate from
-- currency to points lives here; the auto-earn subscriber will read it once
-- publishers carry sale.completed payloads with totals (v1.1).
--
-- See modules/loyalty.

CREATE TABLE IF NOT EXISTS loyalty_programs (
    id                       UUID PRIMARY KEY,
    store_id                 UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    name                     VARCHAR(128) NOT NULL,
    description              TEXT NULL,
    points_per_currency_unit NUMERIC(20, 4) NOT NULL DEFAULT 1,
    -- NULL means points never expire; otherwise points expire after N days.
    expiration_days          INTEGER NULL,
    is_active                BOOLEAN NOT NULL DEFAULT TRUE,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT loyalty_programs_store_name_unique UNIQUE (store_id, name),
    CONSTRAINT loyalty_programs_rate_positive
        CHECK (points_per_currency_unit > 0),
    CONSTRAINT loyalty_programs_expiration_positive
        CHECK (expiration_days IS NULL OR expiration_days > 0)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_programs_store
    ON loyalty_programs (store_id) WHERE is_active = TRUE;
