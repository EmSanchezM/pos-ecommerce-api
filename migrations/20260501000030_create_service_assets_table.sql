-- Service orders: customer-owned things being serviced.
--
-- An asset belongs to a store; `customer_id` is nullable because a walk-in
-- intake can capture a brand/model/identifier before the customer record
-- exists. v1.0 supports five types: vehicle, equipment, appliance,
-- electronic, other.
--
-- The "service history" of an asset is just `SELECT * FROM service_orders
-- WHERE asset_id = X` — we don't keep a separate visit table.
--
-- See modules/service_orders.

CREATE TABLE IF NOT EXISTS service_assets (
    id          UUID PRIMARY KEY,
    store_id    UUID NOT NULL REFERENCES stores(id)    ON DELETE CASCADE,
    customer_id UUID NULL     REFERENCES customers(id) ON DELETE SET NULL,
    asset_type  VARCHAR(16) NOT NULL,
    brand       VARCHAR(80) NULL,
    model       VARCHAR(120) NULL,
    identifier  VARCHAR(120) NULL,           -- license plate, serial, IMEI
    year        INTEGER NULL,
    color       VARCHAR(40) NULL,
    description TEXT NULL,
    attributes  JSONB NOT NULL DEFAULT '{}'::JSONB,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT service_assets_type_chk
        CHECK (asset_type IN ('vehicle', 'equipment', 'appliance', 'electronic', 'other')),
    CONSTRAINT service_assets_year_chk
        CHECK (year IS NULL OR (year BETWEEN 1900 AND 2100))
);

CREATE INDEX IF NOT EXISTS idx_service_assets_store_active
    ON service_assets (store_id)
    WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_service_assets_customer
    ON service_assets (customer_id)
    WHERE customer_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_service_assets_identifier
    ON service_assets (store_id, identifier)
    WHERE identifier IS NOT NULL;
