-- Loyalty: points ledger
--
-- Append-only audit row. `points` is signed: positive for earn / positive
-- adjustment, negative for redeem / expire / negative adjustment. Sum of all
-- entries for a member equals their current balance (the cached
-- `loyalty_members.current_balance` mirrors this).
--
-- See modules/loyalty.

CREATE TABLE IF NOT EXISTS loyalty_points_ledger (
    id            UUID PRIMARY KEY,
    member_id     UUID NOT NULL REFERENCES loyalty_members(id) ON DELETE RESTRICT,
    txn_type      VARCHAR(16) NOT NULL,
    points        BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,
    source_type   VARCHAR(32) NULL,
    source_id     UUID NULL,
    occurred_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMPTZ NULL,
    reason        TEXT NULL,
    created_by    UUID NULL REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT loyalty_points_ledger_type_check
        CHECK (txn_type IN ('earn', 'redeem', 'expire', 'adjustment')),
    CONSTRAINT loyalty_points_ledger_points_nonzero CHECK (points <> 0)
);

-- Hot path: list a member's history newest-first.
CREATE INDEX IF NOT EXISTS idx_loyalty_points_ledger_member
    ON loyalty_points_ledger (member_id, occurred_at DESC);

-- Hot path for the expiration job: find earn rows whose expires_at has come
-- due. Partial index keeps it small.
CREATE INDEX IF NOT EXISTS idx_loyalty_points_ledger_expirable
    ON loyalty_points_ledger (member_id, expires_at)
    WHERE txn_type = 'earn' AND expires_at IS NOT NULL;
