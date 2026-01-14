-- Migration: Create audit_log table
-- Audit logging for all identity-related changes

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    actor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Reference to the user who performed the action
    CONSTRAINT fk_audit_log_actor
        FOREIGN KEY (actor_id) 
        REFERENCES users (id) 
        ON DELETE SET NULL
);

-- Indexes for faster lookups and filtering
CREATE INDEX IF NOT EXISTS idx_audit_log_entity_type ON audit_log (entity_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity_id ON audit_log (entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor_id ON audit_log (actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log (created_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log (action);

-- Composite index for date range queries with entity filtering
CREATE INDEX IF NOT EXISTS idx_audit_log_date_range ON audit_log (created_at, entity_type, entity_id);
