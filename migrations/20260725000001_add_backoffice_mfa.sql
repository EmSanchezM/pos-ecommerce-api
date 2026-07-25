-- Migration: 20260725000001_add_backoffice_mfa
--
-- Completes the MFA storage that 20260602000001 only reserved a column for.
--
-- `backoffice_users.mfa_secret` has existed since the initial migration but
-- nothing read it, and on its own it cannot express enrollment state: a secret
-- is generated BEFORE the operator has proven they can produce a valid code.
-- Activating on generation alone would lock an operator out the moment their
-- authenticator failed to scan.

-- NULL until the operator confirms enrollment with a valid code. A row with a
-- secret and no activation timestamp is a pending enrollment, and pending
-- enrollment must never be treated as protection.
ALTER TABLE backoffice_users
    ADD COLUMN IF NOT EXISTS mfa_activated_at TIMESTAMPTZ NULL;

-- The last TOTP step counter accepted for this user. A code stays valid for its
-- whole window, so without this a code observed in transit can be replayed
-- until the window closes. Verification must reject any step <= this value.
ALTER TABLE backoffice_users
    ADD COLUMN IF NOT EXISTS mfa_last_used_step BIGINT NULL;

-- Single-use recovery codes, so losing the authenticator device does not mean
-- losing platform-owner access permanently. There is no backoffice user-admin
-- endpoint that could reset MFA for someone.
CREATE TABLE IF NOT EXISTS backoffice_mfa_recovery_codes (
    id                  UUID        PRIMARY KEY,
    backoffice_user_id  UUID        NOT NULL
        REFERENCES backoffice_users (id) ON DELETE CASCADE,
    -- Argon2 hash, never the code itself. A leaked table must not yield a
    -- working second factor.
    code_hash           VARCHAR(255) NOT NULL,
    -- Set when consumed. Rows are retained rather than deleted so the audit
    -- trail can show that a recovery path was used, and when.
    used_at             TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Verification loads a user's unused codes on every recovery attempt.
CREATE INDEX IF NOT EXISTS idx_backoffice_mfa_recovery_codes_user
    ON backoffice_mfa_recovery_codes (backoffice_user_id)
    WHERE used_at IS NULL;
