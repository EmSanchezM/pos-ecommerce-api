-- Migration: Create product_categories table
-- Hierarchical product categorization with parent-child relationships

CREATE TABLE IF NOT EXISTS product_categories (
    id UUID PRIMARY KEY,
    parent_id UUID NULL REFERENCES product_categories(id) ON DELETE SET NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT NULL,
    slug VARCHAR(100) NOT NULL,
    icon VARCHAR(100) NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT product_categories_slug_unique UNIQUE (slug)
);

-- Index for parent-child lookups
CREATE INDEX IF NOT EXISTS idx_product_categories_parent_id ON product_categories(parent_id);

-- Index for slug lookups (URL-friendly navigation)
CREATE INDEX IF NOT EXISTS idx_product_categories_slug ON product_categories(slug);

-- Index for active categories
CREATE INDEX IF NOT EXISTS idx_product_categories_is_active ON product_categories(is_active);

-- Index for sorting within each level
CREATE INDEX IF NOT EXISTS idx_product_categories_sort_order ON product_categories(parent_id, sort_order);
