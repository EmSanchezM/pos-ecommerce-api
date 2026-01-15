-- Migration: Create inventory_reservations table
-- Temporary stock holds for shopping carts with expiration

CREATE TABLE IF NOT EXISTS inventory_reservations (
    id UUID PRIMARY KEY,
    stock_id UUID NOT NULL REFERENCES inventory_stock(id) ON DELETE CASCADE,
    reference_type VARCHAR(50) NOT NULL,
    reference_id UUID NOT NULL,
    quantity NUMERIC(20, 4) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Status must be one of the valid values
    CONSTRAINT inventory_reservations_status_check CHECK (
        status IN ('pending', 'confirmed', 'cancelled', 'expired')
    ),

    -- Quantity must be positive
    CONSTRAINT inventory_reservations_quantity_check CHECK (quantity > 0)
);

-- Index for stock lookups
CREATE INDEX IF NOT EXISTS idx_inventory_reservations_stock_id ON inventory_reservations(stock_id);

-- Index for reference lookups
CREATE INDEX IF NOT EXISTS idx_inventory_reservations_reference ON inventory_reservations(reference_type, reference_id);

-- Index for expiration queries (find expired pending reservations)
CREATE INDEX IF NOT EXISTS idx_inventory_reservations_expires_at ON inventory_reservations(expires_at)
    WHERE status = 'pending';

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_inventory_reservations_status ON inventory_reservations(status);

-- Composite index for finding expired pending reservations efficiently
CREATE INDEX IF NOT EXISTS idx_inventory_reservations_pending_expires ON inventory_reservations(status, expires_at)
    WHERE status = 'pending';
