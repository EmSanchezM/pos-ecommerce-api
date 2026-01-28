-- Create vendors table for supplier/vendor management
-- Part of the Purchasing module

CREATE TABLE IF NOT EXISTS vendors (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255) NOT NULL,
    tax_id VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    address TEXT,
    payment_terms_days INTEGER NOT NULL DEFAULT 30,
    currency CHAR(3) NOT NULL DEFAULT 'HNL',
    is_active BOOLEAN NOT NULL DEFAULT true,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT vendors_code_unique UNIQUE (code),
    CONSTRAINT vendors_tax_id_unique UNIQUE (tax_id),
    CONSTRAINT vendors_payment_terms_positive CHECK (payment_terms_days >= 0)
);

-- Create indexes for common queries
CREATE INDEX idx_vendors_code ON vendors(code);
CREATE INDEX idx_vendors_name ON vendors(name);
CREATE INDEX idx_vendors_tax_id ON vendors(tax_id);
CREATE INDEX idx_vendors_is_active ON vendors(is_active);

-- Add comments for documentation
COMMENT ON TABLE vendors IS 'Suppliers/vendors for the purchasing system';
COMMENT ON COLUMN vendors.code IS 'Unique vendor code (e.g., PROV-001)';
COMMENT ON COLUMN vendors.name IS 'Commercial name of the vendor';
COMMENT ON COLUMN vendors.legal_name IS 'Legal/registered name of the vendor';
COMMENT ON COLUMN vendors.tax_id IS 'Tax identification number (RTN in Honduras)';
COMMENT ON COLUMN vendors.payment_terms_days IS 'Default payment terms in days (e.g., 30 for Net 30)';
COMMENT ON COLUMN vendors.currency IS 'Default currency for transactions with this vendor (ISO 4217)';
