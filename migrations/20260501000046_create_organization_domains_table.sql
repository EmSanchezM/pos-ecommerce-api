-- Tenancy: custom domains pointing at an organization (e.g. `tienda.acme.com`
-- → org X). v1.0 records the domain + a verification token but does NOT
-- check DNS — admins call POST .../verify manually. v1.1 adds a job that
-- looks up the DNS TXT record and marks `verified_at` automatically.
--
-- The partial unique index forces at most one `is_primary = TRUE` per org;
-- the application's `set_primary` helper transactionally clears + sets to
-- maintain the invariant.

CREATE TABLE IF NOT EXISTS organization_domains (
    id                  UUID PRIMARY KEY,
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain              VARCHAR(253) NOT NULL UNIQUE,
    is_verified         BOOLEAN NOT NULL DEFAULT FALSE,
    is_primary          BOOLEAN NOT NULL DEFAULT FALSE,
    verification_token  VARCHAR(64) NULL,
    verified_at         TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_organization_domains_org
    ON organization_domains (organization_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_organization_domains_one_primary
    ON organization_domains (organization_id)
    WHERE is_primary = TRUE;
