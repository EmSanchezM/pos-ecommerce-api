-- Wishlist: one per (customer, store). Items deduplicated per (wishlist, listing).

CREATE TABLE IF NOT EXISTS wishlists (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES customers(id),
    store_id UUID NOT NULL REFERENCES stores(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wishlists_customer_store_unique UNIQUE (customer_id, store_id)
);

CREATE INDEX IF NOT EXISTS idx_wishlists_customer ON wishlists(customer_id);

CREATE TABLE IF NOT EXISTS wishlist_items (
    id UUID PRIMARY KEY,
    wishlist_id UUID NOT NULL REFERENCES wishlists(id) ON DELETE CASCADE,
    listing_id UUID NOT NULL REFERENCES product_listings(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wishlist_items_unique UNIQUE (wishlist_id, listing_id)
);

CREATE INDEX IF NOT EXISTS idx_wishlist_items_wishlist
    ON wishlist_items(wishlist_id, added_at DESC);
