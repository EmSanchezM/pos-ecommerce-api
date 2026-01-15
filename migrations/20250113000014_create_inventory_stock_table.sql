-- Migration: Create inventory_stock table
-- Stock per location with optimistic locking and XOR constraint

CREATE TABLE IF NOT EXISTS inventory_stock (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE RESTRICT,
    product_id UUID NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    quantity NUMERIC(20, 4) NOT NULL DEFAULT 0,
    reserved_quantity NUMERIC(20, 4) NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    min_stock_level NUMERIC(20, 4) NOT NULL DEFAULT 0,
    max_stock_level NUMERIC(20, 4) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- XOR constraint: exactly one of product_id or variant_id must be set
    CONSTRAINT inventory_stock_product_variant_xor CHECK (
        (product_id IS NOT NULL AND variant_id IS NULL) OR
        (product_id IS NULL AND variant_id IS NOT NULL)
    ),

    -- Reserved quantity cannot exceed total quantity
    CONSTRAINT inventory_stock_reserved_check CHECK (reserved_quantity <= quantity),

    -- Quantity cannot be negative
    CONSTRAINT inventory_stock_quantity_check CHECK (quantity >= 0),

    -- Unique stock per store/product or store/variant
    CONSTRAINT inventory_stock_store_product_unique UNIQUE (store_id, product_id),
    CONSTRAINT inventory_stock_store_variant_unique UNIQUE (store_id, variant_id)
);

-- Generated column for available quantity (quantity - reserved_quantity)
-- Note: PostgreSQL doesn't support generated columns in all versions, so we compute this in queries

-- Index for store lookups
CREATE INDEX IF NOT EXISTS idx_inventory_stock_store_id ON inventory_stock(store_id);

-- Index for product lookups
CREATE INDEX IF NOT EXISTS idx_inventory_stock_product_id ON inventory_stock(product_id) WHERE product_id IS NOT NULL;

-- Index for variant lookups
CREATE INDEX IF NOT EXISTS idx_inventory_stock_variant_id ON inventory_stock(variant_id) WHERE variant_id IS NOT NULL;

-- Index for low stock alerts (available_quantity <= min_stock_level)
CREATE INDEX IF NOT EXISTS idx_inventory_stock_low_stock ON inventory_stock(store_id, min_stock_level)
    WHERE (quantity - reserved_quantity) <= min_stock_level;
