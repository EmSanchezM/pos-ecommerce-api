-- Tenancy: per-org branding (logo, favicon, color palette, theme, custom
-- CSS). PK = FK to organizations; one row per org, written via upsert.

CREATE TABLE IF NOT EXISTS organization_branding (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    logo_url        VARCHAR(500) NULL,
    favicon_url     VARCHAR(500) NULL,
    primary_color   VARCHAR(7)   NULL,
    secondary_color VARCHAR(7)   NULL,
    accent_color    VARCHAR(7)   NULL,
    theme           VARCHAR(16)  NOT NULL DEFAULT 'system',
    custom_css      TEXT         NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT organization_branding_theme_chk
        CHECK (theme IN ('light', 'dark', 'system'))
);
