-- Public/eCommerce-facing layer over inventory.products.
-- A listing is the SKU as seen by the shopper: slug, descriptions, SEO,
-- featured flag, view count.

CREATE TABLE IF NOT EXISTS product_listings (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    product_id UUID NOT NULL REFERENCES products(id),
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    short_description TEXT,
    long_description TEXT,
    is_published BOOLEAN NOT NULL DEFAULT false,
    is_featured BOOLEAN NOT NULL DEFAULT false,
    seo_title VARCHAR(255),
    seo_description TEXT,
    seo_keywords TEXT[] NOT NULL DEFAULT '{}',
    sort_order INT NOT NULL DEFAULT 0,
    view_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT product_listings_store_slug_unique UNIQUE (store_id, slug),
    CONSTRAINT product_listings_product_unique UNIQUE (product_id)
);

CREATE INDEX IF NOT EXISTS idx_listings_store_id ON product_listings(store_id);
CREATE INDEX IF NOT EXISTS idx_listings_product_id ON product_listings(product_id);
CREATE INDEX IF NOT EXISTS idx_listings_published_featured
    ON product_listings(store_id, is_published, is_featured)
    WHERE is_published = true;
-- Trigram index for fuzzy search on title (extension required).
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_listings_title_trgm
    ON product_listings USING gin (title gin_trgm_ops);
