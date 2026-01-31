-- Create carts table for e-commerce shopping carts
CREATE TABLE IF NOT EXISTS carts (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    customer_id UUID REFERENCES customers(id),
    session_id VARCHAR(255), -- For anonymous carts
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    subtotal DECIMAL(15,4) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    total DECIMAL(15,4) NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    converted_to_sale BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_carts_store_id ON carts(store_id);
CREATE INDEX idx_carts_customer_id ON carts(customer_id);
CREATE INDEX idx_carts_session_id ON carts(session_id);
CREATE INDEX idx_carts_expires_at ON carts(expires_at);
CREATE INDEX idx_carts_converted ON carts(converted_to_sale);

-- Partial index for active carts
CREATE INDEX idx_carts_active
    ON carts(store_id, customer_id)
    WHERE converted_to_sale = FALSE AND expires_at > NOW();
