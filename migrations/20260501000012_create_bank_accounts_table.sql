-- Cash management: bank_accounts
--
-- One row per (store, bank, account_number). `current_balance` is the *book*
-- balance (what we believe the bank holds based on entries we recorded), not
-- the live bank balance — those values diverge until reconciliation.
-- `version` enables optimistic locking on concurrent transaction posts.
--
-- See modules/cash_management.

CREATE TABLE IF NOT EXISTS bank_accounts (
    id              UUID PRIMARY KEY,
    store_id        UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    bank_name       VARCHAR(100) NOT NULL,
    account_number  VARCHAR(64) NOT NULL,
    account_type    VARCHAR(16) NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'HNL',
    current_balance NUMERIC(20, 4) NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    version         INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT bank_accounts_account_type_check
        CHECK (account_type IN ('checking', 'savings', 'other')),
    CONSTRAINT bank_accounts_account_number_unique UNIQUE (account_number)
);

CREATE INDEX IF NOT EXISTS idx_bank_accounts_store
    ON bank_accounts (store_id) WHERE is_active = TRUE;
