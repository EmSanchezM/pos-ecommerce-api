-- Migration: Create permissions table
-- Requirements: 1.1 - Permission management with unique code format module:action

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT permissions_code_unique UNIQUE (code),
    -- Validate format: must contain exactly one colon with non-empty parts
    CONSTRAINT permissions_code_format CHECK (
        code ~ '^[a-z_]+:[a-z_]+$'
    )
);

-- Index for faster lookups by code and module prefix
CREATE INDEX IF NOT EXISTS idx_permissions_code ON permissions (code);
