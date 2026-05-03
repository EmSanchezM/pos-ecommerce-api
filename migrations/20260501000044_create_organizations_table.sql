-- Tenancy: organization (tenant) — top-level container for stores, users,
-- plan, branding, and custom domains. The CHECK on `slug` mirrors the domain
-- validator in `Organization::register` so wrong slugs fail at the API edge
-- with a clean code instead of a 23514 leaking from the DB.
--
-- See modules/tenancy.

CREATE TABLE IF NOT EXISTS organizations (
    id            UUID PRIMARY KEY,
    name          VARCHAR(160) NOT NULL,
    slug          VARCHAR(60)  NOT NULL UNIQUE,
    contact_email VARCHAR(160) NOT NULL,
    contact_phone VARCHAR(40)  NULL,
    status        VARCHAR(16)  NOT NULL DEFAULT 'active',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT organizations_status_chk
        CHECK (status IN ('active', 'suspended', 'pending_setup')),
    CONSTRAINT organizations_slug_chk
        CHECK (slug ~ '^[a-z0-9][a-z0-9-]{1,58}[a-z0-9]$')
);

CREATE INDEX IF NOT EXISTS idx_organizations_status
    ON organizations (status);
