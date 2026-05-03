-- Tenancy: data migration that creates a deterministic "default"
-- organization and backfills every existing user and store to it.
--
-- The default org id is hard-coded so the seed binary (which runs *after*
-- migrations) can refer to it without an extra lookup. Slug `default`
-- satisfies the slug CHECK constraint. The plan defaults to `enterprise`
-- with all feature flags ON so existing single-tenant deployments keep
-- access to every module.
--
-- This migration is idempotent (the INSERTs use ON CONFLICT DO NOTHING and
-- the UPDATEs only touch rows where organization_id IS NULL).

DO $$
DECLARE
    default_org_id UUID := '00000000-0000-0000-0000-000000000001';
BEGIN
    INSERT INTO organizations (
        id, name, slug, contact_email, contact_phone, status,
        created_at, updated_at
    )
    VALUES (
        default_org_id,
        'Organización Por Defecto',
        'default',
        'admin@example.com',
        NULL,
        'active',
        NOW(),
        NOW()
    )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO organization_plans (
        id, organization_id, tier, feature_flags,
        seat_limit, store_limit, starts_at, expires_at,
        created_at, updated_at
    )
    VALUES (
        gen_random_uuid(),
        default_org_id,
        'enterprise',
        jsonb_build_object(
            'booking',         TRUE,
            'restaurant',      TRUE,
            'service_orders',  TRUE,
            'loyalty',         TRUE
        ),
        NULL, NULL, NOW(), NULL,
        NOW(), NOW()
    )
    ON CONFLICT (organization_id) DO NOTHING;

    -- Backfill: assign every existing user / store to the default org. Only
    -- touches rows where the column is currently NULL so re-runs are safe.
    UPDATE users
       SET organization_id = default_org_id
     WHERE organization_id IS NULL;

    UPDATE stores
       SET organization_id = default_org_id
     WHERE organization_id IS NULL;
END $$;
