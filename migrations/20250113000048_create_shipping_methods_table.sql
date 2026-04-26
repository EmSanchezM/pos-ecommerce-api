-- Shipping methods catalog (per store).
--
-- method_type values:
--   store_pickup       - customer collects at the store
--   own_delivery       - delivered by a store-owned driver
--   external_delivery  - third-party courier (Hugo, PedidosYa, Uber, Servientrega)
--   standard, express, same_day, free_shipping  - generic carrier-agnostic

CREATE TABLE IF NOT EXISTS shipping_methods (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) NOT NULL,
    method_type VARCHAR(30) NOT NULL,
    description TEXT,
    estimated_days_min INT,
    estimated_days_max INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT shipping_methods_store_code_unique UNIQUE (store_id, code)
);

CREATE INDEX IF NOT EXISTS idx_shipping_methods_store_id ON shipping_methods(store_id);
CREATE INDEX IF NOT EXISTS idx_shipping_methods_active ON shipping_methods(is_active);
