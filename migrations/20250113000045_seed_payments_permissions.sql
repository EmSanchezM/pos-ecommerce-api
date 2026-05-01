-- Permission rows for the payments module.
--
-- payment_gateways:* permissions exist to keep the format consistent and to
-- preserve them in audit logs, BUT mutating routes (create/update/delete) are
-- gated by the `system:admin` super-admin check at the handler layer. Listing
-- and reading gateways requires `payment_gateways:read`.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'payment_gateways:read',    'Read payment gateway configuration'),
    (gen_random_uuid(), 'payment_gateways:create',  'Create payment gateway (super_admin only)'),
    (gen_random_uuid(), 'payment_gateways:update',  'Update payment gateway (super_admin only)'),
    (gen_random_uuid(), 'payment_gateways:delete',  'Delete payment gateway (super_admin only)'),
    (gen_random_uuid(), 'transactions:create',      'Process online payment transactions'),
    (gen_random_uuid(), 'transactions:read',        'Read payment transactions'),
    (gen_random_uuid(), 'transactions:refund',      'Refund payment transactions'),
    (gen_random_uuid(), 'transactions:reconcile',   'Reconcile payment transactions against gateway'),
    (gen_random_uuid(), 'payouts:read',             'Read gateway payouts/settlements')
ON CONFLICT (code) DO NOTHING;
