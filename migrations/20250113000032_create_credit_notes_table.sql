-- Create credit_notes table
CREATE TABLE IF NOT EXISTS credit_notes (
    id UUID PRIMARY KEY,
    credit_note_number VARCHAR(50) NOT NULL,
    store_id UUID NOT NULL REFERENCES stores(id),
    original_sale_id UUID NOT NULL REFERENCES sales(id),
    original_invoice_number VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft, pending, approved, applied, cancelled
    return_type VARCHAR(20) NOT NULL, -- full, partial
    return_reason VARCHAR(50) NOT NULL, -- defective, wrong_item, not_as_described, changed_mind, etc.
    reason_details TEXT,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    subtotal DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    total DECIMAL(15,4) NOT NULL DEFAULT 0,
    refund_method VARCHAR(50), -- cash, original_payment, store_credit
    refunded_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    created_by_id UUID NOT NULL REFERENCES users(id),
    submitted_by_id UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    approved_by_id UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    applied_by_id UUID REFERENCES users(id),
    applied_at TIMESTAMPTZ,
    cancelled_by_id UUID REFERENCES users(id),
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT credit_notes_store_number_unique UNIQUE (store_id, credit_note_number)
);

-- Indexes
CREATE INDEX idx_credit_notes_store_id ON credit_notes(store_id);
CREATE INDEX idx_credit_notes_original_sale_id ON credit_notes(original_sale_id);
CREATE INDEX idx_credit_notes_status ON credit_notes(status);
CREATE INDEX idx_credit_notes_created_by_id ON credit_notes(created_by_id);
CREATE INDEX idx_credit_notes_created_at ON credit_notes(created_at);
