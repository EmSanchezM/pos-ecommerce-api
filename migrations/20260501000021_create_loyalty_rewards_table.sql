-- Loyalty: rewards + redemptions
--
-- `loyalty_rewards` is the catalog of redeemable items. `reward_value` is the
-- magnitude (50 = L 50, 10 = 10 %); the `reward_type` decides interpretation.
-- `loyalty_reward_redemptions` is the voucher row created when a member
-- spends points; `applied_to_sale_id` is set later by the storefront once the
-- discount is consumed.
--
-- See modules/loyalty.

CREATE TABLE IF NOT EXISTS loyalty_rewards (
    id                          UUID PRIMARY KEY,
    program_id                  UUID NOT NULL REFERENCES loyalty_programs(id) ON DELETE CASCADE,
    name                        VARCHAR(128) NOT NULL,
    description                 TEXT NULL,
    cost_points                 BIGINT NOT NULL,
    reward_type                 VARCHAR(32) NOT NULL,
    reward_value                NUMERIC(20, 4) NOT NULL,
    max_redemptions_per_member  INTEGER NULL,
    is_active                   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT loyalty_rewards_type_check
        CHECK (reward_type IN ('discount_amount', 'discount_percent', 'free_product')),
    CONSTRAINT loyalty_rewards_cost_positive CHECK (cost_points > 0),
    CONSTRAINT loyalty_rewards_max_positive
        CHECK (max_redemptions_per_member IS NULL OR max_redemptions_per_member > 0)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_rewards_program
    ON loyalty_rewards (program_id) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS loyalty_reward_redemptions (
    id                  UUID PRIMARY KEY,
    member_id           UUID NOT NULL REFERENCES loyalty_members(id) ON DELETE RESTRICT,
    reward_id           UUID NOT NULL REFERENCES loyalty_rewards(id) ON DELETE RESTRICT,
    ledger_entry_id     UUID NOT NULL REFERENCES loyalty_points_ledger(id) ON DELETE RESTRICT,
    applied_to_sale_id  UUID NULL REFERENCES sales(id) ON DELETE SET NULL,
    redeemed_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_loyalty_reward_redemptions_member
    ON loyalty_reward_redemptions (member_id, redeemed_at DESC);

CREATE INDEX IF NOT EXISTS idx_loyalty_reward_redemptions_reward
    ON loyalty_reward_redemptions (reward_id);

-- Hot path: count redemptions for the per-member cap check.
CREATE INDEX IF NOT EXISTS idx_loyalty_reward_redemptions_member_reward
    ON loyalty_reward_redemptions (member_id, reward_id);
