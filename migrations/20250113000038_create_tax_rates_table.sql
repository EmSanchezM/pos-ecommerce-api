-- Tax rates table for configurable tax rates per store
-- Supports Honduras ISV (15% general, 18% special) and exempt categories

CREATE TABLE IF NOT EXISTS tax_rates (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    tax_type VARCHAR(20) NOT NULL,         -- 'isv_15', 'isv_18', 'exempt'
    rate DECIMAL(5,4) NOT NULL,            -- 0.1500, 0.1800, 0.0000
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    applies_to VARCHAR(20) NOT NULL DEFAULT 'all',  -- 'all', 'categories'
    category_ids UUID[] DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT tax_rates_store_name_unique UNIQUE (store_id, name),
    CONSTRAINT tax_rates_rate_check CHECK (rate >= 0 AND rate <= 1)
);

CREATE INDEX idx_tax_rates_store_id ON tax_rates(store_id);
CREATE INDEX idx_tax_rates_is_active ON tax_rates(is_active);
