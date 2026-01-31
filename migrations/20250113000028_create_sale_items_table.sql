-- Create sale_items table
CREATE TABLE IF NOT EXISTS sale_items (
    id UUID PRIMARY KEY,
    sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id),
    variant_id UUID REFERENCES product_variants(id),
    sku VARCHAR(100) NOT NULL,
    description VARCHAR(500) NOT NULL,
    quantity DECIMAL(15,4) NOT NULL,
    unit_of_measure VARCHAR(50) NOT NULL,
    unit_price DECIMAL(15,4) NOT NULL,
    unit_cost DECIMAL(15,4) NOT NULL DEFAULT 0,
    discount_type VARCHAR(20), -- 'percentage' or 'fixed'
    discount_value DECIMAL(15,4) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    subtotal DECIMAL(15,4) NOT NULL,
    total DECIMAL(15,4) NOT NULL,
    reservation_id UUID REFERENCES inventory_reservations(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sale_items_sale_id ON sale_items(sale_id);
CREATE INDEX idx_sale_items_product_id ON sale_items(product_id);
CREATE INDEX idx_sale_items_variant_id ON sale_items(variant_id);
CREATE INDEX idx_sale_items_reservation_id ON sale_items(reservation_id);
