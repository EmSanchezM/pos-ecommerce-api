-- Migration: Create user_store_roles table (store-scoped role assignments)
-- Assign roles to users scoped by store_id

CREATE TABLE IF NOT EXISTS user_store_roles (
    user_id UUID NOT NULL,
    store_id UUID NOT NULL,
    role_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (user_id, store_id, role_id),
    
    CONSTRAINT fk_user_store_roles_user
        FOREIGN KEY (user_id) 
        REFERENCES users (id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_user_store_roles_store
        FOREIGN KEY (store_id) 
        REFERENCES stores (id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_user_store_roles_role
        FOREIGN KEY (role_id) 
        REFERENCES roles (id) 
        ON DELETE CASCADE,
    
    -- Ensure user is a member of the store before assigning roles
    CONSTRAINT fk_user_store_roles_membership
        FOREIGN KEY (user_id, store_id) 
        REFERENCES user_stores (user_id, store_id) 
        ON DELETE CASCADE
);

-- Indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_store_roles_user_id ON user_store_roles (user_id);
CREATE INDEX IF NOT EXISTS idx_user_store_roles_store_id ON user_store_roles (store_id);
CREATE INDEX IF NOT EXISTS idx_user_store_roles_role_id ON user_store_roles (role_id);
CREATE INDEX IF NOT EXISTS idx_user_store_roles_user_store ON user_store_roles (user_id, store_id);
