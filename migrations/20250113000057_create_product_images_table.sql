-- Image gallery for a listing.
-- `url` is the public URL the frontend uses; `storage_key` is the
-- adapter-specific identifier the system needs to delete the file (file
-- path for LocalServer, S3 key for S3, public_id for Cloudinary, etc).

CREATE TABLE IF NOT EXISTS product_images (
    id UUID PRIMARY KEY,
    listing_id UUID NOT NULL REFERENCES product_listings(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    storage_key TEXT NOT NULL,
    storage_provider_id UUID,
    alt_text VARCHAR(255),
    sort_order INT NOT NULL DEFAULT 0,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    content_type VARCHAR(100),
    size_bytes BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- One primary image per listing (partial unique index).
CREATE UNIQUE INDEX IF NOT EXISTS product_images_one_primary_per_listing
    ON product_images(listing_id)
    WHERE is_primary = true;

CREATE INDEX IF NOT EXISTS idx_images_listing_sorted
    ON product_images(listing_id, sort_order ASC);
