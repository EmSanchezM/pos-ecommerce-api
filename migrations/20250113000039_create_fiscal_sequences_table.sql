-- Fiscal sequences for invoice number generation
-- Tracks the current correlative number per terminal/CAI combination

CREATE TABLE IF NOT EXISTS fiscal_sequences (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    terminal_id UUID NOT NULL REFERENCES terminals(id),
    cai_range_id UUID NOT NULL REFERENCES cai_ranges(id),
    prefix VARCHAR(50) NOT NULL,           -- e.g., '000-001-01-'
    current_number BIGINT NOT NULL DEFAULT 0,
    range_start BIGINT NOT NULL,
    range_end BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fiscal_seq_terminal_cai_unique UNIQUE (terminal_id, cai_range_id),
    CONSTRAINT fiscal_seq_range_check CHECK (range_start <= range_end),
    CONSTRAINT fiscal_seq_current_check CHECK (current_number >= 0)
);

CREATE INDEX idx_fiscal_sequences_terminal ON fiscal_sequences(terminal_id);
CREATE INDEX idx_fiscal_sequences_active ON fiscal_sequences(is_active);
