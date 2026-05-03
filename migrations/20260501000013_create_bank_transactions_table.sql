-- Cash management: bank_transactions
--
-- One row per statement line we've recorded. `amount` follows accounting
-- convention (positive = inflow, negative = outflow); the `txn_type` field
-- says why. The domain rejects type/sign mismatches except for `adjustment`
-- which the accountant uses for corrections.
--
-- See modules/cash_management.

CREATE TABLE IF NOT EXISTS bank_transactions (
    id                UUID PRIMARY KEY,
    bank_account_id   UUID NOT NULL REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    txn_type          VARCHAR(16) NOT NULL,
    amount            NUMERIC(20, 4) NOT NULL,
    reference         VARCHAR(128) NULL,
    description       TEXT NULL,
    occurred_at       TIMESTAMPTZ NOT NULL,
    reconciled        BOOLEAN NOT NULL DEFAULT FALSE,
    -- bank_reconciliations is created in the next migration; this FK is added
    -- there with ALTER TABLE so we don't take a forward-reference dependency.
    reconciliation_id UUID NULL,
    created_by        UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT bank_transactions_type_check
        CHECK (txn_type IN (
            'deposit', 'withdrawal', 'fee', 'interest',
            'transfer_in', 'transfer_out', 'adjustment'
        ))
);

CREATE INDEX IF NOT EXISTS idx_bank_transactions_account
    ON bank_transactions (bank_account_id, occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_bank_transactions_unreconciled
    ON bank_transactions (bank_account_id) WHERE reconciled = FALSE;

CREATE INDEX IF NOT EXISTS idx_bank_transactions_reconciliation
    ON bank_transactions (reconciliation_id) WHERE reconciliation_id IS NOT NULL;
