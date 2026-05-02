-- Cash management: cash_deposits
--
-- Manager records that money from a closed `cashier_shift` is being moved to
-- the bank. Workflow `pending → deposited → reconciled`. One deposit per
-- shift (UNIQUE constraint).
--
-- See modules/cash_management.

CREATE TABLE IF NOT EXISTS cash_deposits (
    id                    UUID PRIMARY KEY,
    cashier_shift_id      UUID NOT NULL REFERENCES cashier_shifts(id) ON DELETE RESTRICT,
    bank_account_id       UUID NOT NULL REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    amount                NUMERIC(20, 4) NOT NULL,
    deposit_date          DATE NOT NULL,
    deposit_slip_number   VARCHAR(64) NULL,
    deposited_by_user_id  UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    bank_transaction_id   UUID NULL REFERENCES bank_transactions(id) ON DELETE SET NULL,
    status                VARCHAR(16) NOT NULL DEFAULT 'pending',
    notes                 TEXT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT cash_deposits_status_check
        CHECK (status IN ('pending', 'deposited', 'reconciled')),
    CONSTRAINT cash_deposits_amount_positive CHECK (amount > 0),
    CONSTRAINT cash_deposits_shift_unique UNIQUE (cashier_shift_id)
);

CREATE INDEX IF NOT EXISTS idx_cash_deposits_account
    ON cash_deposits (bank_account_id, deposit_date DESC);

CREATE INDEX IF NOT EXISTS idx_cash_deposits_status
    ON cash_deposits (status) WHERE status <> 'reconciled';

CREATE INDEX IF NOT EXISTS idx_cash_deposits_bank_transaction
    ON cash_deposits (bank_transaction_id) WHERE bank_transaction_id IS NOT NULL;
