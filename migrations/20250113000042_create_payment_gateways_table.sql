-- Payment gateways: per-store gateway configuration.
-- Mutating these rows is restricted to super admins at the API layer; the table
-- itself is permission-agnostic.

CREATE TABLE IF NOT EXISTS payment_gateways (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    gateway_type VARCHAR(30) NOT NULL,           -- stripe, paypal, bac_credomatic, ficohsa, manual
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    api_key_encrypted TEXT NOT NULL,
    secret_key_encrypted TEXT NOT NULL,
    merchant_id VARCHAR(100),
    is_sandbox BOOLEAN NOT NULL DEFAULT false,
    supported_methods TEXT[] NOT NULL DEFAULT '{}',
    supported_currencies VARCHAR(3)[] NOT NULL DEFAULT '{HNL}',
    webhook_secret TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT payment_gateways_store_name_unique UNIQUE (store_id, name)
);

-- Single default per store, only when active.
CREATE UNIQUE INDEX IF NOT EXISTS payment_gateways_one_default_per_store
    ON payment_gateways (store_id)
    WHERE is_default = true;

CREATE INDEX IF NOT EXISTS idx_payment_gateways_store_id ON payment_gateways(store_id);
CREATE INDEX IF NOT EXISTS idx_payment_gateways_is_active ON payment_gateways(is_active);
CREATE INDEX IF NOT EXISTS idx_payment_gateways_gateway_type ON payment_gateways(gateway_type);
