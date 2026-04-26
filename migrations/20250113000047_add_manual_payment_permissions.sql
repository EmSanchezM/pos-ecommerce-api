-- Permissions for the manual-confirmation flow.
--
-- transactions:confirm is required to manually confirm or reject a pending
-- transaction (typically a manager who reconciles the bank statement).

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'transactions:confirm', 'Manually confirm/reject pending payment transactions')
ON CONFLICT (code) DO NOTHING;
