-- Invoice lines table for individual line items within an invoice
-- Each line represents a product/service with its tax calculation

CREATE TABLE IF NOT EXISTS invoice_lines (
    id UUID PRIMARY KEY,
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    product_id UUID NOT NULL,
    variant_id UUID,
    sku VARCHAR(100) NOT NULL,
    description VARCHAR(500) NOT NULL,
    quantity DECIMAL(15,4) NOT NULL,
    unit_of_measure VARCHAR(20) NOT NULL,
    unit_price DECIMAL(15,4) NOT NULL,
    discount_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_type VARCHAR(20) NOT NULL,         -- 'isv_15', 'isv_18', 'exempt'
    tax_rate DECIMAL(5,4) NOT NULL,
    tax_amount DECIMAL(15,4) NOT NULL,
    subtotal DECIMAL(15,4) NOT NULL,
    total DECIMAL(15,4) NOT NULL,
    is_exempt BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoice_lines_invoice_id ON invoice_lines(invoice_id);
