-- Migration: Create product_variants table
-- Product variations (size, color, flavor) with CASCADE delete

CREATE TABLE IF NOT EXISTS product_variants (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(50) NOT NULL,
    barcode VARCHAR(100) NULL,
    name VARCHAR(255) NOT NULL,
    variant_attributes JSONB NOT NULL DEFAULT '{}',
    price NUMERIC(20, 4) NULL,
    cost_price NUMERIC(20, 4) NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT product_variants_sku_unique UNIQUE (sku),
    CONSTRAINT product_variants_barcode_unique UNIQUE (barcode)
);

-- Index for product lookups
CREATE INDEX IF NOT EXISTS idx_product_variants_product_id ON product_variants(product_id);

-- Index for SKU lookups
CREATE INDEX IF NOT EXISTS idx_product_variants_sku ON product_variants(sku);

-- Index for barcode lookups
CREATE INDEX IF NOT EXISTS idx_product_variants_barcode ON product_variants(barcode) WHERE barcode IS NOT NULL;

-- Index for active variants
CREATE INDEX IF NOT EXISTS idx_product_variants_is_active ON product_variants(is_active);
