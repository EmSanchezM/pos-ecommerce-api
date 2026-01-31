-- Create customers table
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    customer_type VARCHAR(20) NOT NULL DEFAULT 'individual',
    code VARCHAR(50) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    company_name VARCHAR(200),
    email VARCHAR(255),
    phone VARCHAR(50),
    tax_id VARCHAR(50),
    -- Billing address fields
    address_line1 VARCHAR(255),
    address_line2 VARCHAR(255),
    address_city VARCHAR(100),
    address_state VARCHAR(100),
    address_postal_code VARCHAR(20),
    address_country VARCHAR(100),
    -- Link to user account for e-commerce
    user_id UUID REFERENCES users(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    -- Statistics
    total_purchases DECIMAL(15,4) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0,
    last_purchase_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT customers_store_code_unique UNIQUE (store_id, code),
    CONSTRAINT customers_store_email_unique UNIQUE (store_id, email)
);

-- Indexes
CREATE INDEX idx_customers_store_id ON customers(store_id);
CREATE INDEX idx_customers_user_id ON customers(user_id);
CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_customers_is_active ON customers(is_active);
CREATE INDEX idx_customers_search ON customers(first_name, last_name, company_name);
