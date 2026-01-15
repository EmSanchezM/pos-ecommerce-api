-- Migration: Create products table
-- Main product catalog with SKU, pricing, and attributes

CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY,
    sku VARCHAR(50) NOT NULL,
    barcode VARCHAR(100) NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    category_id UUID NULL REFERENCES product_categories(id) ON DELETE SET NULL,
    brand VARCHAR(100) NULL,
    unit_of_measure VARCHAR(20) NOT NULL,
    base_price NUMERIC(20, 4) NOT NULL DEFAULT 0,
    cost_price NUMERIC(20, 4) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    is_perishable BOOLEAN NOT NULL DEFAULT FALSE,
    is_trackable BOOLEAN NOT NULL DEFAULT TRUE,
    has_variants BOOLEAN NOT NULL DEFAULT FALSE,
    tax_rate NUMERIC(5, 4) NOT NULL DEFAULT 0,
    tax_included BOOLEAN NOT NULL DEFAULT FALSE,
    attributes JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT products_sku_unique UNIQUE (sku),
    CONSTRAINT products_barcode_unique UNIQUE (barcode)
);

-- Index for SKU lookups
CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku);

-- Index for barcode lookups
CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode) WHERE barcode IS NOT NULL;

-- Index for category filtering
CREATE INDEX IF NOT EXISTS idx_products_category_id ON products(category_id);

-- Index for active products
CREATE INDEX IF NOT EXISTS idx_products_is_active ON products(is_active);

-- Index for products with variants
CREATE INDEX IF NOT EXISTS idx_products_has_variants ON products(has_variants) WHERE has_variants = TRUE;
