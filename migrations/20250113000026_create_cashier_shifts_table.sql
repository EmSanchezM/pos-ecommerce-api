-- Create cashier_shifts table
CREATE TABLE IF NOT EXISTS cashier_shifts (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    terminal_id UUID NOT NULL REFERENCES terminals(id),
    cashier_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    opened_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    opening_balance DECIMAL(15,4) NOT NULL DEFAULT 0,
    closing_balance DECIMAL(15,4),
    expected_balance DECIMAL(15,4) NOT NULL DEFAULT 0,
    cash_sales DECIMAL(15,4) NOT NULL DEFAULT 0,
    card_sales DECIMAL(15,4) NOT NULL DEFAULT 0,
    other_sales DECIMAL(15,4) NOT NULL DEFAULT 0,
    refunds DECIMAL(15,4) NOT NULL DEFAULT 0,
    cash_in DECIMAL(15,4) NOT NULL DEFAULT 0,
    cash_out DECIMAL(15,4) NOT NULL DEFAULT 0,
    transaction_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    closing_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_cashier_shifts_store_id ON cashier_shifts(store_id);
CREATE INDEX idx_cashier_shifts_terminal_id ON cashier_shifts(terminal_id);
CREATE INDEX idx_cashier_shifts_cashier_id ON cashier_shifts(cashier_id);
CREATE INDEX idx_cashier_shifts_status ON cashier_shifts(status);
CREATE INDEX idx_cashier_shifts_opened_at ON cashier_shifts(opened_at);

-- Partial index for open shifts (only one open shift per terminal/cashier)
CREATE UNIQUE INDEX idx_cashier_shifts_terminal_open
    ON cashier_shifts(terminal_id)
    WHERE status = 'open';

CREATE UNIQUE INDEX idx_cashier_shifts_cashier_open
    ON cashier_shifts(cashier_id)
    WHERE status = 'open';
