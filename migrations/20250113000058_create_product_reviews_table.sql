-- Customer reviews per listing. One review per (listing, customer).
-- `is_verified_purchase` is set automatically when the customer has a
-- completed sale containing the underlying product.
-- `is_approved` defaults to false — manager approves before public display.

CREATE TABLE IF NOT EXISTS product_reviews (
    id UUID PRIMARY KEY,
    listing_id UUID NOT NULL REFERENCES product_listings(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id),
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    title VARCHAR(255),
    comment TEXT,
    is_verified_purchase BOOLEAN NOT NULL DEFAULT false,
    is_approved BOOLEAN NOT NULL DEFAULT false,
    approved_by_id UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT product_reviews_listing_customer_unique UNIQUE (listing_id, customer_id)
);

CREATE INDEX IF NOT EXISTS idx_reviews_listing_approved
    ON product_reviews(listing_id, is_approved)
    WHERE is_approved = true;
CREATE INDEX IF NOT EXISTS idx_reviews_pending
    ON product_reviews(created_at DESC)
    WHERE is_approved = false;
