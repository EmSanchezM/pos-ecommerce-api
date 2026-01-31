-- Create payments table
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY,
    sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    payment_method VARCHAR(30) NOT NULL, -- cash, credit_card, debit_card, bank_transfer, paypal, store_credit, gift_card, other
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- pending, completed, failed, refunded, partially_refunded
    amount DECIMAL(15,4) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    amount_tendered DECIMAL(15,4), -- For cash payments
    change_given DECIMAL(15,4), -- For cash payments
    reference_number VARCHAR(100), -- Transaction reference
    authorization_code VARCHAR(100), -- Card authorization
    card_last_four VARCHAR(4), -- Last 4 digits of card
    card_brand VARCHAR(30), -- visa, mastercard, etc.
    refunded_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    refunded_at TIMESTAMPTZ,
    notes TEXT,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_payments_sale_id ON payments(sale_id);
CREATE INDEX idx_payments_payment_method ON payments(payment_method);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_processed_at ON payments(processed_at);
CREATE INDEX idx_payments_reference_number ON payments(reference_number);
