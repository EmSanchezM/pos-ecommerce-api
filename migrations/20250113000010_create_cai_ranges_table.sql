-- Migration: Create cai_ranges table
-- CAI range entity for fiscal document authorization

CREATE TABLE IF NOT EXISTS cai_ranges (
    id UUID PRIMARY KEY,
    terminal_id UUID NOT NULL REFERENCES terminals(id) ON DELETE CASCADE,
    cai_number VARCHAR(50) NOT NULL,
    range_start BIGINT NOT NULL,
    range_end BIGINT NOT NULL,
    current_number BIGINT NOT NULL,
    expiration_date DATE NOT NULL,
    is_exhausted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_cai_valid_range CHECK (range_start <= range_end),
    CONSTRAINT chk_cai_valid_current CHECK (current_number >= range_start AND current_number <= range_end + 1)
);

-- Index for faster lookups by terminal
CREATE INDEX IF NOT EXISTS idx_cai_ranges_terminal_id ON cai_ranges(terminal_id);

-- Index for expiration date queries (e.g., finding soon-to-expire CAIs)
CREATE INDEX IF NOT EXISTS idx_cai_ranges_expiration_date ON cai_ranges(expiration_date);

-- Index for finding active (non-exhausted) CAI ranges
CREATE INDEX IF NOT EXISTS idx_cai_ranges_is_exhausted ON cai_ranges(is_exhausted);
