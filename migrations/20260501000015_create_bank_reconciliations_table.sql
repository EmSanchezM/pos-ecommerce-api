-- Cash management: bank_reconciliations
--
-- Periodic reconciliation between book balance and bank statement balance.
-- `in_progress → completed`. Once completed, every bank_transactions row in
-- [period_start, period_end] for the account is flagged reconciled with this
-- reconciliation id (handled by the use case at close time).
--
-- See modules/cash_management.

CREATE TABLE IF NOT EXISTS bank_reconciliations (
    id                    UUID PRIMARY KEY,
    bank_account_id       UUID NOT NULL REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    period_start          TIMESTAMPTZ NOT NULL,
    period_end            TIMESTAMPTZ NOT NULL,
    opening_balance       NUMERIC(20, 4) NOT NULL,
    closing_book_balance  NUMERIC(20, 4) NULL,
    statement_balance     NUMERIC(20, 4) NULL,
    status                VARCHAR(16) NOT NULL DEFAULT 'in_progress',
    completed_at          TIMESTAMPTZ NULL,
    completed_by          UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    notes                 TEXT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT bank_reconciliations_status_check
        CHECK (status IN ('in_progress', 'completed')),
    CONSTRAINT bank_reconciliations_range_check CHECK (period_start < period_end)
);

CREATE INDEX IF NOT EXISTS idx_bank_reconciliations_account
    ON bank_reconciliations (bank_account_id, period_start DESC);

-- Wire the deferred FK that bank_transactions.reconciliation_id needs.
-- `ON DELETE SET NULL` so rolling back a reconciliation doesn't cascade-
-- delete statement lines, which would lose audit history.
ALTER TABLE bank_transactions
    ADD CONSTRAINT bank_transactions_reconciliation_fkey
    FOREIGN KEY (reconciliation_id)
    REFERENCES bank_reconciliations(id)
    ON DELETE SET NULL;
