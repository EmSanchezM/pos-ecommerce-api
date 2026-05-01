-- Payouts: settlements pulled from each gateway.

CREATE TABLE IF NOT EXISTS payouts (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    gateway_id UUID NOT NULL REFERENCES payment_gateways(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, in_transit, paid, failed, cancelled
    amount DECIMAL(15,4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    fee_amount DECIMAL(15,4) NOT NULL DEFAULT 0,
    net_amount DECIMAL(15,4) NOT NULL,
    gateway_payout_id VARCHAR(255),
    transaction_count INT NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    expected_arrival TIMESTAMPTZ,
    arrived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payouts_store_id ON payouts(store_id);
CREATE INDEX IF NOT EXISTS idx_payouts_gateway_id ON payouts(gateway_id);
CREATE INDEX IF NOT EXISTS idx_payouts_status ON payouts(status);
CREATE INDEX IF NOT EXISTS idx_payouts_period_end ON payouts(period_end);
