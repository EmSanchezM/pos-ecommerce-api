-- Create credit_note_items table
CREATE TABLE IF NOT EXISTS credit_note_items (
    id UUID PRIMARY KEY,
    credit_note_id UUID NOT NULL REFERENCES credit_notes(id) ON DELETE CASCADE,
    original_sale_item_id UUID NOT NULL REFERENCES sale_items(id),
    product_id UUID NOT NULL REFERENCES products(id),
    variant_id UUID REFERENCES product_variants(id),
    sku VARCHAR(100) NOT NULL,
    description VARCHAR(500) NOT NULL,
    return_quantity DECIMAL(15,4) NOT NULL,
    unit_of_measure VARCHAR(50) NOT NULL,
    unit_price DECIMAL(15,4) NOT NULL,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    subtotal DECIMAL(15,4) NOT NULL,
    total DECIMAL(15,4) NOT NULL,
    restock BOOLEAN NOT NULL DEFAULT TRUE,
    condition VARCHAR(50), -- new, used, damaged
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_credit_note_items_credit_note_id ON credit_note_items(credit_note_id);
CREATE INDEX idx_credit_note_items_original_sale_item_id ON credit_note_items(original_sale_item_id);
CREATE INDEX idx_credit_note_items_product_id ON credit_note_items(product_id);
CREATE INDEX idx_credit_note_items_variant_id ON credit_note_items(variant_id);
