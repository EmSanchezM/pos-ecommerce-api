-- Accounting module tables: chart of accounts, accounting periods, and
-- balanced journal entries with their lines. See modules/accounting.

-- =============================================================================
-- chart_of_accounts
-- =============================================================================

CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id           UUID PRIMARY KEY,
    code         VARCHAR(32) NOT NULL,
    name         VARCHAR(255) NOT NULL,
    account_type VARCHAR(16) NOT NULL,
    parent_id    UUID NULL REFERENCES chart_of_accounts(id) ON DELETE SET NULL,
    is_active    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chart_of_accounts_code_unique UNIQUE (code),
    CONSTRAINT chart_of_accounts_type_check
        CHECK (account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense'))
);

CREATE INDEX IF NOT EXISTS idx_chart_of_accounts_type ON chart_of_accounts (account_type);
CREATE INDEX IF NOT EXISTS idx_chart_of_accounts_active
    ON chart_of_accounts (is_active) WHERE is_active = TRUE;

-- =============================================================================
-- accounting_periods
-- =============================================================================

CREATE TABLE IF NOT EXISTS accounting_periods (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL,
    fiscal_year INTEGER NOT NULL,
    starts_at   TIMESTAMPTZ NOT NULL,
    ends_at     TIMESTAMPTZ NOT NULL,
    status      VARCHAR(16) NOT NULL DEFAULT 'open',
    closed_at   TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT accounting_periods_range_check CHECK (starts_at < ends_at),
    CONSTRAINT accounting_periods_status_check CHECK (status IN ('open', 'closed'))
);

CREATE INDEX IF NOT EXISTS idx_accounting_periods_fiscal_year
    ON accounting_periods (fiscal_year);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_range
    ON accounting_periods (starts_at, ends_at);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_status
    ON accounting_periods (status);

-- =============================================================================
-- journal_entries
-- =============================================================================

CREATE TABLE IF NOT EXISTS journal_entries (
    id            UUID PRIMARY KEY,
    period_id     UUID NOT NULL REFERENCES accounting_periods(id) ON DELETE RESTRICT,
    entry_number  BIGINT NOT NULL,
    description   TEXT NOT NULL,
    source_type   VARCHAR(64) NULL,
    source_id     UUID NULL,
    status        VARCHAR(16) NOT NULL DEFAULT 'draft',
    posted_at     TIMESTAMPTZ NULL,
    created_by    UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT journal_entries_status_check
        CHECK (status IN ('draft', 'posted', 'voided')),
    CONSTRAINT journal_entries_period_number_unique UNIQUE (period_id, entry_number)
);

CREATE INDEX IF NOT EXISTS idx_journal_entries_period ON journal_entries (period_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_source
    ON journal_entries (source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_status_period
    ON journal_entries (status, period_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_posted_at
    ON journal_entries (posted_at);

-- =============================================================================
-- journal_lines
-- =============================================================================

CREATE TABLE IF NOT EXISTS journal_lines (
    id                UUID PRIMARY KEY,
    journal_entry_id  UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id        UUID NOT NULL REFERENCES chart_of_accounts(id) ON DELETE RESTRICT,
    store_id          UUID NULL REFERENCES stores(id) ON DELETE SET NULL,
    line_number       INTEGER NOT NULL,
    debit             NUMERIC(20, 4) NOT NULL DEFAULT 0,
    credit            NUMERIC(20, 4) NOT NULL DEFAULT 0,
    description       TEXT NULL,

    -- Each line books to exactly one side; the domain rejects rows that don't
    -- satisfy this, the constraint is the database-level safety net.
    CONSTRAINT journal_lines_amounts_nonneg CHECK (debit >= 0 AND credit >= 0),
    CONSTRAINT journal_lines_one_side CHECK (
        (debit > 0 AND credit = 0) OR (debit = 0 AND credit > 0)
    ),
    CONSTRAINT journal_lines_entry_line_unique UNIQUE (journal_entry_id, line_number)
);

CREATE INDEX IF NOT EXISTS idx_journal_lines_account ON journal_lines (account_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_store
    ON journal_lines (store_id) WHERE store_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_journal_lines_entry ON journal_lines (journal_entry_id);
