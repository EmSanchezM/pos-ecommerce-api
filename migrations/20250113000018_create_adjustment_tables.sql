-- Migration: Create adjustment tables
-- Stock adjustments with approval workflow and CASCADE delete on items

-- Stock Adjustments table
CREATE TABLE IF NOT EXISTS stock_adjustments (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    adjustment_number VARCHAR(50) NOT NULL,
    adjustment_type VARCHAR(20) NOT NULL,
    adjustment_reason VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    created_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    approved_by_id UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    approved_at TIMESTAMPTZ NULL,
    applied_at TIMESTAMPTZ NULL,
    notes TEXT NULL,
    attachments JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Adjustment type must be one of the valid values
    CONSTRAINT stock_adjustments_type_check CHECK (
        adjustment_type IN ('increase', 'decrease')
    ),

    -- Adjustment reason must be one of the valid values
    CONSTRAINT stock_adjustments_reason_check CHECK (
        adjustment_reason IN ('damage', 'theft', 'loss', 'found', 'correction', 'expiration')
    ),

    -- Status must be one of the valid values
    CONSTRAINT stock_adjustments_status_check CHECK (
        status IN ('draft', 'pending_approval', 'approved', 'rejected', 'applied')
    ),

    -- Unique adjustment number per store
    CONSTRAINT stock_adjustments_number_store_unique UNIQUE (store_id, adjustment_number)
);

-- Index for store lookups
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_store_id ON stock_adjustments(store_id);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_status ON stock_adjustments(status);

-- Index for creator lookups
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_created_by ON stock_adjustments(created_by_id);

-- Index for approver lookups
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_approved_by ON stock_adjustments(approved_by_id)
    WHERE approved_by_id IS NOT NULL;

-- Index for pending approvals
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_pending ON stock_adjustments(store_id, status)
    WHERE status = 'pending_approval';


-- Stock Adjustment Items table
CREATE TABLE IF NOT EXISTS stock_adjustment_items (
    id UUID PRIMARY KEY,
    adjustment_id UUID NOT NULL REFERENCES stock_adjustments(id) ON DELETE CASCADE,
    stock_id UUID NOT NULL REFERENCES inventory_stock(id) ON DELETE RESTRICT,
    quantity NUMERIC(20, 4) NOT NULL,
    unit_cost NUMERIC(20, 4) NULL,
    balance_before NUMERIC(20, 4) NULL,
    balance_after NUMERIC(20, 4) NULL,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for adjustment lookups
CREATE INDEX IF NOT EXISTS idx_stock_adjustment_items_adjustment_id ON stock_adjustment_items(adjustment_id);

-- Index for stock lookups
CREATE INDEX IF NOT EXISTS idx_stock_adjustment_items_stock_id ON stock_adjustment_items(stock_id);
