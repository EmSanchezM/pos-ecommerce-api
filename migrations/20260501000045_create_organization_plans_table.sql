-- Tenancy: organization plan (one row per org). `feature_flags` is a flat
-- JSON object the application toggles per-feature. v1.0 reads/writes the
-- flags but does not enforce them; v1.1 will add a `RequireFeature`
-- middleware in the gateway that checks the active org's flags before
-- routing to the relevant module.

CREATE TABLE IF NOT EXISTS organization_plans (
    id              UUID PRIMARY KEY,
    organization_id UUID NOT NULL UNIQUE
                          REFERENCES organizations(id) ON DELETE CASCADE,
    tier            VARCHAR(16) NOT NULL DEFAULT 'free',
    feature_flags   JSONB       NOT NULL DEFAULT '{}'::JSONB,
    seat_limit      INTEGER     NULL,
    store_limit     INTEGER     NULL,
    starts_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT organization_plans_tier_chk
        CHECK (tier IN ('free', 'pro', 'enterprise')),
    CONSTRAINT organization_plans_seats_chk
        CHECK (seat_limit IS NULL OR seat_limit > 0),
    CONSTRAINT organization_plans_stores_chk
        CHECK (store_limit IS NULL OR store_limit > 0)
);
