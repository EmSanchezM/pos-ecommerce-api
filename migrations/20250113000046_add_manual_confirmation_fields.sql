-- Manual payment confirmation flow.
--
-- For Honduras-style payments (transferencia, depósito en agencia, contra
-- entrega) the gateway adapter cannot autoconfirm: a human verifies the
-- deposit in the bank statement and confirms (or rejects) the transaction.

ALTER TABLE payment_transactions
    ADD COLUMN IF NOT EXISTS reference_number VARCHAR(100),
    ADD COLUMN IF NOT EXISTS confirmed_by_id  UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS confirmed_at     TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejected_by_id   UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS rejected_at      TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejection_reason TEXT;

-- Reconciliation searches by reference_number; case-insensitive btree via
-- a simple lower() expression index.
CREATE INDEX IF NOT EXISTS idx_payment_transactions_reference_number
    ON payment_transactions (lower(reference_number))
    WHERE reference_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_payment_transactions_pending_status
    ON payment_transactions (status, store_id)
    WHERE status = 'pending';
