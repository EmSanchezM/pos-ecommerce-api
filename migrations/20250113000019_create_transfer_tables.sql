-- Migration: Create transfer tables
-- Stock transfers between stores with shipping workflow and CASCADE delete on items

-- Stock Transfers table
CREATE TABLE IF NOT EXISTS stock_transfers (
    id UUID PRIMARY KEY,
    transfer_number VARCHAR(50) NOT NULL,
    from_store_id UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    to_store_id UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    requested_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    shipped_date TIMESTAMPTZ NULL,
    received_date TIMESTAMPTZ NULL,
    requested_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    shipped_by_id UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    received_by_id UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    notes TEXT NULL,
    shipping_method VARCHAR(100) NULL,
    tracking_number VARCHAR(100) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Cannot transfer to same store
    CONSTRAINT stock_transfers_different_stores CHECK (from_store_id != to_store_id),

    -- Status must be one of the valid values
    CONSTRAINT stock_transfers_status_check CHECK (
        status IN ('draft', 'pending', 'in_transit', 'completed', 'cancelled')
    ),

    -- Globally unique transfer number
    CONSTRAINT stock_transfers_number_unique UNIQUE (transfer_number)
);

-- Index for source store lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfers_from_store ON stock_transfers(from_store_id);

-- Index for destination store lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfers_to_store ON stock_transfers(to_store_id);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_stock_transfers_status ON stock_transfers(status);

-- Index for requester lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfers_requested_by ON stock_transfers(requested_by_id);

-- Index for shipper lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfers_shipped_by ON stock_transfers(shipped_by_id)
    WHERE shipped_by_id IS NOT NULL;

-- Index for receiver lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfers_received_by ON stock_transfers(received_by_id)
    WHERE received_by_id IS NOT NULL;

-- Index for in-transit transfers
CREATE INDEX IF NOT EXISTS idx_stock_transfers_in_transit ON stock_transfers(status)
    WHERE status = 'in_transit';


-- Stock Transfer Items table
CREATE TABLE IF NOT EXISTS stock_transfer_items (
    id UUID PRIMARY KEY,
    transfer_id UUID NOT NULL REFERENCES stock_transfers(id) ON DELETE CASCADE,
    product_id UUID NULL REFERENCES products(id) ON DELETE RESTRICT,
    variant_id UUID NULL REFERENCES product_variants(id) ON DELETE RESTRICT,
    quantity_requested NUMERIC(20, 4) NOT NULL,
    quantity_shipped NUMERIC(20, 4) NULL,
    quantity_received NUMERIC(20, 4) NULL,
    unit_cost NUMERIC(20, 4) NULL,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- XOR constraint: exactly one of product_id or variant_id must be set
    CONSTRAINT stock_transfer_items_product_variant_xor CHECK (
        (product_id IS NOT NULL AND variant_id IS NULL) OR
        (product_id IS NULL AND variant_id IS NOT NULL)
    ),

    -- Quantity requested must be positive
    CONSTRAINT stock_transfer_items_quantity_check CHECK (quantity_requested > 0)
);

-- Index for transfer lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfer_items_transfer_id ON stock_transfer_items(transfer_id);

-- Index for product lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfer_items_product_id ON stock_transfer_items(product_id)
    WHERE product_id IS NOT NULL;

-- Index for variant lookups
CREATE INDEX IF NOT EXISTS idx_stock_transfer_items_variant_id ON stock_transfer_items(variant_id)
    WHERE variant_id IS NOT NULL;
