-- Create sales table
CREATE TABLE IF NOT EXISTS sales (
    id UUID PRIMARY KEY,
    sale_number VARCHAR(50) NOT NULL,
    store_id UUID NOT NULL REFERENCES stores(id),
    sale_type VARCHAR(20) NOT NULL, -- 'pos' or 'online'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    order_status VARCHAR(30), -- For online orders: pending_payment, paid, processing, shipped, delivered, cancelled, returned
    terminal_id UUID REFERENCES terminals(id),
    shift_id UUID REFERENCES cashier_shifts(id),
    cashier_id UUID REFERENCES users(id),
    customer_id UUID REFERENCES customers(id),
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    subtotal DECIMAL(15,4) NOT NULL DEFAULT 0,
    discount_type VARCHAR(20), -- 'percentage' or 'fixed'
    discount_value DECIMAL(15,4) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    total DECIMAL(15,4) NOT NULL DEFAULT 0,
    amount_paid DECIMAL(15,4) NOT NULL DEFAULT 0,
    amount_due DECIMAL(15,4) NOT NULL DEFAULT 0,
    change_given DECIMAL(15,4) NOT NULL DEFAULT 0,
    invoice_number VARCHAR(50),
    invoice_date TIMESTAMPTZ,
    notes TEXT,
    internal_notes TEXT,
    voided_by_id UUID REFERENCES users(id),
    voided_at TIMESTAMPTZ,
    void_reason TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT sales_store_number_unique UNIQUE (store_id, sale_number)
);

-- Indexes
CREATE INDEX idx_sales_store_id ON sales(store_id);
CREATE INDEX idx_sales_sale_type ON sales(sale_type);
CREATE INDEX idx_sales_status ON sales(status);
CREATE INDEX idx_sales_order_status ON sales(order_status);
CREATE INDEX idx_sales_terminal_id ON sales(terminal_id);
CREATE INDEX idx_sales_shift_id ON sales(shift_id);
CREATE INDEX idx_sales_cashier_id ON sales(cashier_id);
CREATE INDEX idx_sales_customer_id ON sales(customer_id);
CREATE INDEX idx_sales_invoice_number ON sales(invoice_number);
CREATE INDEX idx_sales_created_at ON sales(created_at);
CREATE INDEX idx_sales_completed_at ON sales(completed_at);
