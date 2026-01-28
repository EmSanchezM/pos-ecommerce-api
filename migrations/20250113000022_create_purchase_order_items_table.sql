-- Create purchase_order_items table for line items in purchase orders
-- Part of the Purchasing module

CREATE TABLE IF NOT EXISTS purchase_order_items (
    id UUID PRIMARY KEY,
    purchase_order_id UUID NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id),
    variant_id UUID REFERENCES product_variants(id),
    description VARCHAR(500) NOT NULL,
    quantity_ordered DECIMAL(18, 4) NOT NULL,
    quantity_received DECIMAL(18, 4) NOT NULL DEFAULT 0,
    unit_of_measure VARCHAR(20) NOT NULL,
    unit_cost DECIMAL(18, 4) NOT NULL,
    discount_percent DECIMAL(5, 2) NOT NULL DEFAULT 0,
    tax_percent DECIMAL(5, 2) NOT NULL DEFAULT 0,
    line_total DECIMAL(18, 4) NOT NULL,
    notes TEXT,

    CONSTRAINT po_items_order_line_unique UNIQUE (purchase_order_id, line_number),
    CONSTRAINT po_items_quantity_ordered_positive CHECK (quantity_ordered > 0),
    CONSTRAINT po_items_quantity_received_non_negative CHECK (quantity_received >= 0),
    CONSTRAINT po_items_unit_cost_non_negative CHECK (unit_cost >= 0),
    CONSTRAINT po_items_discount_percent_range CHECK (discount_percent >= 0 AND discount_percent <= 100),
    CONSTRAINT po_items_tax_percent_range CHECK (tax_percent >= 0 AND tax_percent <= 100),
    CONSTRAINT po_items_line_total_non_negative CHECK (line_total >= 0),
    CONSTRAINT po_items_received_not_exceed_ordered CHECK (quantity_received <= quantity_ordered)
);

-- Create indexes for common queries
CREATE INDEX idx_po_items_purchase_order_id ON purchase_order_items(purchase_order_id);
CREATE INDEX idx_po_items_product_id ON purchase_order_items(product_id);
CREATE INDEX idx_po_items_variant_id ON purchase_order_items(variant_id);

-- Add comments for documentation
COMMENT ON TABLE purchase_order_items IS 'Line items in purchase orders';
COMMENT ON COLUMN purchase_order_items.line_number IS 'Sequential line number within the order';
COMMENT ON COLUMN purchase_order_items.quantity_ordered IS 'Original quantity ordered';
COMMENT ON COLUMN purchase_order_items.quantity_received IS 'Total quantity received across all receipts';
COMMENT ON COLUMN purchase_order_items.line_total IS 'Calculated total: (qty * cost - discount%) + tax%';
