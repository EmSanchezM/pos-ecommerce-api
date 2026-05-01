-- Shipping zones: geographic areas the store delivers to.
-- Match precedence (resolved in code): zip_codes > states > countries.

CREATE TABLE IF NOT EXISTS shipping_zones (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    countries TEXT[] NOT NULL DEFAULT '{}',
    states TEXT[] NOT NULL DEFAULT '{}',
    zip_codes TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shipping_zones_store_id ON shipping_zones(store_id);
