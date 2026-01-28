-- Create goods_receipt_items table for line items in goods receipts
-- Part of the Purchasing module

CREATE TABLE IF NOT EXISTS goods_receipt_items (
    id UUID PRIMARY KEY,
    goods_receipt_id UUID NOT NULL REFERENCES goods_receipts(id) ON DELETE CASCADE,
    purchase_order_item_id UUID NOT NULL REFERENCES purchase_order_items(id),
    product_id UUID NOT NULL REFERENCES products(id),
    variant_id UUID REFERENCES product_variants(id),
    quantity_received DECIMAL(18, 4) NOT NULL,
    unit_cost DECIMAL(18, 4) NOT NULL,
    lot_number VARCHAR(100),
    expiry_date DATE,
    notes TEXT,

    CONSTRAINT gr_items_quantity_received_positive CHECK (quantity_received > 0),
    CONSTRAINT gr_items_unit_cost_non_negative CHECK (unit_cost >= 0)
);

-- Create indexes for common queries
CREATE INDEX idx_gr_items_goods_receipt_id ON goods_receipt_items(goods_receipt_id);
CREATE INDEX idx_gr_items_purchase_order_item_id ON goods_receipt_items(purchase_order_item_id);
CREATE INDEX idx_gr_items_product_id ON goods_receipt_items(product_id);
CREATE INDEX idx_gr_items_variant_id ON goods_receipt_items(variant_id);
CREATE INDEX idx_gr_items_lot_number ON goods_receipt_items(lot_number);

-- Add comments for documentation
COMMENT ON TABLE goods_receipt_items IS 'Line items in goods receipts';
COMMENT ON COLUMN goods_receipt_items.purchase_order_item_id IS 'Reference to the original purchase order item';
COMMENT ON COLUMN goods_receipt_items.lot_number IS 'Lot/batch number for traceability';
COMMENT ON COLUMN goods_receipt_items.expiry_date IS 'Expiration date for perishable goods';
