-- Composite indexes for common query patterns (Fase 1 - Item 1.7)

CREATE INDEX IF NOT EXISTS idx_sales_store_status ON sales(store_id, status);
CREATE INDEX IF NOT EXISTS idx_sales_store_shift ON sales(store_id, shift_id);
CREATE INDEX IF NOT EXISTS idx_sales_draft_pending ON sales(store_id) WHERE status IN ('draft', 'pending');
CREATE INDEX IF NOT EXISTS idx_reservations_ref_status ON inventory_reservations(reference_type, reference_id, status);
