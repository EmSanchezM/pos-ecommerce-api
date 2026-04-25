-- Invoices table for fiscal electronic invoicing
-- Supports standard invoices, credit notes, debit notes, and proformas
-- Compliant with Honduras SAR (Servicio de Administracion de Rentas) requirements

CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY,
    invoice_number VARCHAR(50) NOT NULL,
    store_id UUID NOT NULL REFERENCES stores(id),
    terminal_id UUID NOT NULL REFERENCES terminals(id),
    sale_id UUID NOT NULL REFERENCES sales(id),
    cai_range_id UUID NOT NULL REFERENCES cai_ranges(id),
    invoice_type VARCHAR(20) NOT NULL,     -- 'standard', 'credit_note', 'debit_note', 'proforma'
    status VARCHAR(20) NOT NULL DEFAULT 'emitted',  -- 'draft', 'emitted', 'voided', 'cancelled'
    customer_id UUID REFERENCES customers(id),
    customer_name VARCHAR(255) NOT NULL,
    customer_rtn VARCHAR(20),              -- Registro Tributario Nacional
    customer_address TEXT,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    subtotal DECIMAL(15,4) NOT NULL,
    exempt_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    taxable_amount_15 DECIMAL(15,4) NOT NULL DEFAULT 0,
    taxable_amount_18 DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_15 DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_18 DECIMAL(15,4) NOT NULL DEFAULT 0,
    total_tax DECIMAL(15,4) NOT NULL,
    discount_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    total DECIMAL(15,4) NOT NULL,
    amount_in_words VARCHAR(500) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    cai_number VARCHAR(50) NOT NULL,
    cai_expiry_date TIMESTAMPTZ NOT NULL,
    range_start VARCHAR(50) NOT NULL,
    range_end VARCHAR(50) NOT NULL,
    voided_by_id UUID REFERENCES users(id),
    voided_at TIMESTAMPTZ,
    void_reason TEXT,
    void_invoice_id UUID REFERENCES invoices(id),
    original_invoice_id UUID REFERENCES invoices(id),
    printed_at TIMESTAMPTZ,
    emitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT invoices_number_unique UNIQUE (store_id, invoice_number)
);

CREATE INDEX idx_invoices_store_id ON invoices(store_id);
CREATE INDEX idx_invoices_terminal_id ON invoices(terminal_id);
CREATE INDEX idx_invoices_sale_id ON invoices(sale_id);
CREATE INDEX idx_invoices_invoice_type ON invoices(invoice_type);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_customer_rtn ON invoices(customer_rtn);
CREATE INDEX idx_invoices_emitted_at ON invoices(emitted_at);
