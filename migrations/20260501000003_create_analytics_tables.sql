-- Analytics module tables: KPI snapshots, dashboards, dashboard widgets.
-- Cross-module aggregations are computed on demand against existing tables
-- (sales, sale_items, products, inventory_stock, users) so no materialized
-- views are introduced here. The recompute job upserts canonical KPIs into
-- `kpi_snapshots`; dashboards read snapshots back at view time.
-- See modules/analytics.

-- =============================================================================
-- kpi_snapshots
-- =============================================================================

CREATE TABLE IF NOT EXISTS kpi_snapshots (
    id           UUID PRIMARY KEY,
    kpi_key      VARCHAR(128) NOT NULL,
    store_id     UUID NULL REFERENCES stores(id) ON DELETE CASCADE,
    time_window  VARCHAR(32) NOT NULL,
    value        NUMERIC(20, 4) NOT NULL,
    metadata     JSONB NOT NULL DEFAULT '{}'::jsonb,
    computed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hot lookup: fetch the latest snapshot for a (kpi_key, store, window) tuple.
CREATE INDEX IF NOT EXISTS idx_kpi_snapshots_lookup
    ON kpi_snapshots (kpi_key, time_window, store_id, computed_at DESC);

-- Listing snapshots per store (dashboard rendering).
CREATE INDEX IF NOT EXISTS idx_kpi_snapshots_store
    ON kpi_snapshots (store_id, computed_at DESC);

-- =============================================================================
-- dashboards
-- =============================================================================

CREATE TABLE IF NOT EXISTS dashboards (
    id             UUID PRIMARY KEY,
    store_id       UUID NULL REFERENCES stores(id) ON DELETE CASCADE,
    owner_user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           VARCHAR(255) NOT NULL,
    description    TEXT NULL,
    layout         JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboards_owner
    ON dashboards (owner_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_dashboards_store
    ON dashboards (store_id) WHERE store_id IS NOT NULL;

-- =============================================================================
-- dashboard_widgets
-- =============================================================================

CREATE TABLE IF NOT EXISTS dashboard_widgets (
    id            UUID PRIMARY KEY,
    dashboard_id  UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    title         VARCHAR(255) NOT NULL,
    kind          VARCHAR(32) NOT NULL,
    kpi_key       VARCHAR(128) NULL,
    time_window   VARCHAR(32) NOT NULL,
    position      INTEGER NOT NULL DEFAULT 0,
    config        JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_dashboard
    ON dashboard_widgets (dashboard_id, position);
