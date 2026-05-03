-- Permissions for the cash_management module: bank accounts, manual bank
-- transactions, cash deposits, and reconciliations. Single-colon format so
-- the permissions_code_format check accepts these.
--
-- All are normal store-level operations; super_admin and store_admin get
-- everything, store_manager gets read + the deposit workflow (they own the
-- physical cash hand-off), but reconciliation closing is admin-only because
-- it's the audit boundary.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'cash_management:read_account',          'Read bank accounts'),
    (gen_random_uuid(), 'cash_management:write_account',         'Create or update bank accounts'),
    (gen_random_uuid(), 'cash_management:read_transaction',      'List bank transactions'),
    (gen_random_uuid(), 'cash_management:write_transaction',     'Record manual bank transactions'),
    (gen_random_uuid(), 'cash_management:read_deposit',          'List cash deposits'),
    (gen_random_uuid(), 'cash_management:write_deposit',         'Create cash deposits and mark them sent to bank'),
    (gen_random_uuid(), 'cash_management:link_deposit',          'Link a deposit to its matching bank transaction'),
    (gen_random_uuid(), 'cash_management:read_reconciliation',   'List bank reconciliations'),
    (gen_random_uuid(), 'cash_management:write_reconciliation',  'Start a bank reconciliation'),
    (gen_random_uuid(), 'cash_management:close_reconciliation',  'Close a bank reconciliation (audit boundary)')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code IN (
      'cash_management:read_account',
      'cash_management:write_account',
      'cash_management:read_transaction',
      'cash_management:write_transaction',
      'cash_management:read_deposit',
      'cash_management:write_deposit',
      'cash_management:link_deposit',
      'cash_management:read_reconciliation',
      'cash_management:write_reconciliation',
      'cash_management:close_reconciliation'
  )
ON CONFLICT DO NOTHING;
