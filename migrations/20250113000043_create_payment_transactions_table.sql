-- Payment transactions: charges, refunds and voids backed by an idempotency key.

CREATE TABLE IF NOT EXISTS payment_transactions (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    gateway_id UUID NOT NULL REFERENCES payment_gateways(id),
    sale_id UUID NOT NULL REFERENCES sales(id),
    payment_id UUID REFERENCES payments(id),
    transaction_type VARCHAR(20) NOT NULL,       -- charge, refund, partial_refund, void
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    amount DECIMAL(15,4) NOT NULL CHECK (amount > 0),
    currency VARCHAR(3) NOT NULL,
    gateway_transaction_id VARCHAR(255),
    gateway_response TEXT,
    authorization_code VARCHAR(100),
    card_last_four VARCHAR(4),
    card_brand VARCHAR(30),
    failure_code VARCHAR(100),
    failure_message TEXT,
    refund_reason TEXT,
    original_transaction_id UUID REFERENCES payment_transactions(id),
    idempotency_key VARCHAR(255) NOT NULL,
    metadata TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT payment_transactions_idempotency_unique UNIQUE (idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_payment_transactions_store_id ON payment_transactions(store_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_sale_id ON payment_transactions(sale_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_gateway_id ON payment_transactions(gateway_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_status ON payment_transactions(status);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_gateway_tx_id
    ON payment_transactions(gateway_transaction_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_created_at
    ON payment_transactions(created_at);
