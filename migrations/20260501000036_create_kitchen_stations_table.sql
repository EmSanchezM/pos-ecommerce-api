-- Restaurant operations: kitchen stations.
--
-- A station is where KDS tickets are routed (Hot Line, Cold Line, Bar). v1.0
-- routing is manual at ticket creation time; v1.1 will add a configurable
-- mapping table from product category to station for auto-routing from sales.

CREATE TABLE IF NOT EXISTS kitchen_stations (
    id         UUID PRIMARY KEY,
    store_id   UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    name       VARCHAR(80) NOT NULL,
    color      VARCHAR(7)  NULL,
    sort_order INTEGER     NOT NULL DEFAULT 0,
    is_active  BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kitchen_stations_store_active
    ON kitchen_stations (store_id, sort_order)
    WHERE is_active = TRUE;
