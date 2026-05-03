-- Tenancy: transversal ALTER. Adds `organization_id` to the two root tables
-- (`users` and `stores`). NULLABLE in v1.0 for backward compatibility — the
-- next data migration creates the default org and backfills every existing
-- row to it. v1.2 will flip these columns to NOT NULL after we've validated
-- the backfill in production.
--
-- Other tables (products, customers, sales, kds_tickets, appointments, etc.)
-- inherit the organization via their FK to `store_id` — no per-table column
-- needed.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS organization_id UUID NULL
    REFERENCES organizations(id) ON DELETE SET NULL;

ALTER TABLE stores
    ADD COLUMN IF NOT EXISTS organization_id UUID NULL
    REFERENCES organizations(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_users_organization
    ON users (organization_id) WHERE organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stores_organization
    ON stores (organization_id) WHERE organization_id IS NOT NULL;
