-- Migration: Create stores table
-- Store management with name, address, is_ecommerce flag

CREATE TABLE IF NOT EXISTS stores (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    is_ecommerce BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_stores_is_active ON stores (is_active);
CREATE INDEX IF NOT EXISTS idx_stores_is_ecommerce ON stores (is_ecommerce);
