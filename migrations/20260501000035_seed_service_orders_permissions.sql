-- Permissions for the service_orders module. Single-colon format respects
-- permissions_code_format. super_admin and store_admin get the full set;
-- public service_orders endpoints (status by token) require no permission.
--
-- IMPORTANT: this is a backup. The seed binary
-- (seed/src/data.rs::PERMISSIONS + ROLE_PERMISSIONS) is the actual source of
-- truth for which roles get which perms, since it runs after migrations.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'service_orders:read_asset',           'List/read assets being serviced'),
    (gen_random_uuid(), 'service_orders:write_asset',          'Register/update/deactivate assets'),
    (gen_random_uuid(), 'service_orders:read_order',           'List/read service orders'),
    (gen_random_uuid(), 'service_orders:write_order',          'Create service orders (intake)'),
    (gen_random_uuid(), 'service_orders:transition_order',     'Diagnose/start-repair/start-testing/mark-ready/deliver an order'),
    (gen_random_uuid(), 'service_orders:cancel_order',         'Cancel a service order'),
    (gen_random_uuid(), 'service_orders:write_item',           'Add/update/remove labor or parts items'),
    (gen_random_uuid(), 'service_orders:write_diagnostic',     'Record technician diagnostics'),
    (gen_random_uuid(), 'service_orders:write_quote',          'Draft a quote from current items'),
    (gen_random_uuid(), 'service_orders:transition_quote',     'Send/approve/reject a quote')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code LIKE 'service_orders:%'
ON CONFLICT DO NOTHING;
