-- Migration: Create terminals table
-- Requirements: 5.1 - Terminal entity with store association

CREATE TABLE IF NOT EXISTS terminals (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_terminals_store_code UNIQUE(store_id, code)
);

-- Index for faster lookups by store
CREATE INDEX IF NOT EXISTS idx_terminals_store_id ON terminals(store_id);

-- Index for filtering by active status
CREATE INDEX IF NOT EXISTS idx_terminals_is_active ON terminals(is_active);
