-- External delivery providers: per-store config for Hugo, PedidosYa, Uber Eats,
-- Servientrega, and Manual (offline coordination — register tracking by hand).
--
-- Mirrors `payment_gateways`: super-admin manages CUD; credentials encrypted
-- at rest. Webhook secret is per-provider.

CREATE TABLE IF NOT EXISTS delivery_providers (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    provider_type VARCHAR(30) NOT NULL,             -- hugo, pedidos_ya, uber_eats, servientrega, manual
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    api_key_encrypted TEXT NOT NULL,
    secret_key_encrypted TEXT NOT NULL,
    merchant_id VARCHAR(100),
    is_sandbox BOOLEAN NOT NULL DEFAULT false,
    coverage_zone_ids UUID[] NOT NULL DEFAULT '{}', -- zones this provider covers
    webhook_secret TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT delivery_providers_store_name_unique UNIQUE (store_id, name)
);

CREATE UNIQUE INDEX IF NOT EXISTS delivery_providers_one_default_per_store
    ON delivery_providers (store_id)
    WHERE is_default = true;

CREATE INDEX IF NOT EXISTS idx_delivery_providers_store_id ON delivery_providers(store_id);
CREATE INDEX IF NOT EXISTS idx_delivery_providers_provider_type
    ON delivery_providers(provider_type);
