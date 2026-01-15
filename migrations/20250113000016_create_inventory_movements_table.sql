-- Migration: Create inventory_movements table
-- Kardex entries for all stock changes with cost tracking

CREATE TABLE IF NOT EXISTS inventory_movements (
    id UUID PRIMARY KEY,
    stock_id UUID NOT NULL REFERENCES inventory_stock(id) ON DELETE CASCADE,
    movement_type VARCHAR(20) NOT NULL,
    movement_reason VARCHAR(100) NULL,
    quantity NUMERIC(20, 4) NOT NULL,
    unit_cost NUMERIC(20, 4) NULL,
    -- Generated column for total_cost (quantity * unit_cost)
    total_cost NUMERIC(20, 4) GENERATED ALWAYS AS (ABS(quantity) * unit_cost) STORED,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    balance_after NUMERIC(20, 4) NOT NULL,
    reference_type VARCHAR(50) NULL,
    reference_id UUID NULL,
    actor_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    notes TEXT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Movement type must be one of the valid values
    CONSTRAINT inventory_movements_type_check CHECK (
        movement_type IN ('in', 'out', 'adjustment', 'transfer_out', 'transfer_in', 'reservation', 'release')
    )
);

-- Index for stock lookups (Kardex queries)
CREATE INDEX IF NOT EXISTS idx_inventory_movements_stock_id ON inventory_movements(stock_id);

-- Index for chronological ordering (Kardex)
CREATE INDEX IF NOT EXISTS idx_inventory_movements_created_at ON inventory_movements(stock_id, created_at DESC);

-- Index for reference lookups
CREATE INDEX IF NOT EXISTS idx_inventory_movements_reference ON inventory_movements(reference_type, reference_id)
    WHERE reference_type IS NOT NULL;

-- Index for movement type filtering
CREATE INDEX IF NOT EXISTS idx_inventory_movements_type ON inventory_movements(movement_type);

-- Index for actor lookups
CREATE INDEX IF NOT EXISTS idx_inventory_movements_actor_id ON inventory_movements(actor_id);

-- Index for weighted average cost calculation (in movements with cost)
CREATE INDEX IF NOT EXISTS idx_inventory_movements_cost_calc ON inventory_movements(stock_id, movement_type, unit_cost)
    WHERE movement_type = 'in' AND unit_cost IS NOT NULL;
