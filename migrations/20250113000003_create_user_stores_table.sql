-- Migration: Create user_stores table (many-to-many relationship)
-- User-Store membership relationship

CREATE TABLE IF NOT EXISTS user_stores (
    user_id UUID NOT NULL,
    store_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (user_id, store_id),
    
    CONSTRAINT fk_user_stores_user
        FOREIGN KEY (user_id) 
        REFERENCES users (id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_user_stores_store
        FOREIGN KEY (store_id) 
        REFERENCES stores (id) 
        ON DELETE CASCADE
);

-- Indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_stores_user_id ON user_stores (user_id);
CREATE INDEX IF NOT EXISTS idx_user_stores_store_id ON user_stores (store_id);
