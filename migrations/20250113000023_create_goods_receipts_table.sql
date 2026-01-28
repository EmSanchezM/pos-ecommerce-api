-- Create goods_receipts table for receiving merchandise
-- Part of the Purchasing module

CREATE TABLE IF NOT EXISTS goods_receipts (
    id UUID PRIMARY KEY,
    receipt_number VARCHAR(50) NOT NULL,
    purchase_order_id UUID NOT NULL REFERENCES purchase_orders(id),
    store_id UUID NOT NULL REFERENCES stores(id),
    receipt_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    notes TEXT,
    received_by_id UUID NOT NULL REFERENCES users(id),
    confirmed_by_id UUID REFERENCES users(id),
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT goods_receipts_receipt_number_store_unique UNIQUE (store_id, receipt_number),
    CONSTRAINT goods_receipts_status_check CHECK (status IN ('draft', 'confirmed', 'cancelled'))
);

-- Create indexes for common queries
CREATE INDEX idx_goods_receipts_purchase_order_id ON goods_receipts(purchase_order_id);
CREATE INDEX idx_goods_receipts_store_id ON goods_receipts(store_id);
CREATE INDEX idx_goods_receipts_status ON goods_receipts(status);
CREATE INDEX idx_goods_receipts_receipt_date ON goods_receipts(receipt_date);
CREATE INDEX idx_goods_receipts_received_by_id ON goods_receipts(received_by_id);

-- Add comments for documentation
COMMENT ON TABLE goods_receipts IS 'Goods receipt documents for receiving merchandise from purchase orders';
COMMENT ON COLUMN goods_receipts.receipt_number IS 'Unique receipt number within the store (e.g., GR-2024-00001)';
COMMENT ON COLUMN goods_receipts.status IS 'Workflow status: draft, confirmed, cancelled';
