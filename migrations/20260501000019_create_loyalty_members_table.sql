-- Loyalty: members
--
-- Links a `customer` to a `program`. `current_balance` and `lifetime_points`
-- cache the running totals so reads don't have to aggregate the ledger.
-- They are bumped atomically alongside ledger inserts (UPDATE ... SET
-- balance = balance + $delta) by the Pg points ledger repository.
--
-- See modules/loyalty.

CREATE TABLE IF NOT EXISTS loyalty_members (
    id                UUID PRIMARY KEY,
    program_id        UUID NOT NULL REFERENCES loyalty_programs(id) ON DELETE RESTRICT,
    customer_id       UUID NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    current_tier_id   UUID NULL REFERENCES loyalty_member_tiers(id) ON DELETE SET NULL,
    current_balance   BIGINT NOT NULL DEFAULT 0,
    lifetime_points   BIGINT NOT NULL DEFAULT 0,
    enrolled_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT loyalty_members_program_customer_unique UNIQUE (program_id, customer_id),
    CONSTRAINT loyalty_members_balance_nonneg CHECK (current_balance >= 0),
    CONSTRAINT loyalty_members_lifetime_nonneg CHECK (lifetime_points >= 0)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_members_program
    ON loyalty_members (program_id, lifetime_points DESC);

CREATE INDEX IF NOT EXISTS idx_loyalty_members_customer
    ON loyalty_members (customer_id);
