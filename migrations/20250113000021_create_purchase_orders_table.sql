-- Create purchase_orders table for purchase order header information
-- Part of the Purchasing module

CREATE TABLE IF NOT EXISTS purchase_orders (
    id UUID PRIMARY KEY,
    order_number VARCHAR(50) NOT NULL,
    store_id UUID NOT NULL REFERENCES stores(id),
    vendor_id UUID NOT NULL REFERENCES vendors(id),
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    order_date DATE NOT NULL,
    expected_delivery_date DATE,
    received_date DATE,
    subtotal DECIMAL(18, 4) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(18, 4) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(18, 4) NOT NULL DEFAULT 0,
    total DECIMAL(18, 4) NOT NULL DEFAULT 0,
    currency CHAR(3) NOT NULL DEFAULT 'HNL',
    payment_terms_days INTEGER NOT NULL DEFAULT 30,
    notes TEXT,
    internal_notes TEXT,
    created_by_id UUID NOT NULL REFERENCES users(id),
    submitted_by_id UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    approved_by_id UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    received_by_id UUID REFERENCES users(id),
    cancelled_by_id UUID REFERENCES users(id),
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT purchase_orders_order_number_store_unique UNIQUE (store_id, order_number),
    CONSTRAINT purchase_orders_status_check CHECK (
        status IN ('draft', 'submitted', 'approved', 'partially_received', 'received', 'closed', 'cancelled')
    ),
    CONSTRAINT purchase_orders_amounts_non_negative CHECK (
        subtotal >= 0 AND tax_amount >= 0 AND discount_amount >= 0 AND total >= 0
    )
);

-- Create indexes for common queries
CREATE INDEX idx_purchase_orders_store_id ON purchase_orders(store_id);
CREATE INDEX idx_purchase_orders_vendor_id ON purchase_orders(vendor_id);
CREATE INDEX idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX idx_purchase_orders_order_date ON purchase_orders(order_date);
CREATE INDEX idx_purchase_orders_created_by_id ON purchase_orders(created_by_id);
CREATE INDEX idx_purchase_orders_order_number ON purchase_orders(order_number);

-- Add comments for documentation
COMMENT ON TABLE purchase_orders IS 'Purchase order header information with workflow status';
COMMENT ON COLUMN purchase_orders.order_number IS 'Unique order number within the store (e.g., PO-2024-00001)';
COMMENT ON COLUMN purchase_orders.status IS 'Workflow status: draft, submitted, approved, partially_received, received, closed, cancelled';
COMMENT ON COLUMN purchase_orders.internal_notes IS 'Internal notes visible only to staff (e.g., rejection reasons)';
