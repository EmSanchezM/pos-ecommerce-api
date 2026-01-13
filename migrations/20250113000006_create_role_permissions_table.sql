-- Migration: Create role_permissions table (many-to-many relationship)
-- Requirements: 2.3 - Assign permissions to roles

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (role_id, permission_id),
    
    CONSTRAINT fk_role_permissions_role
        FOREIGN KEY (role_id) 
        REFERENCES roles (id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_role_permissions_permission
        FOREIGN KEY (permission_id) 
        REFERENCES permissions (id) 
        ON DELETE CASCADE
);

-- Indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions (role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON role_permissions (permission_id);
