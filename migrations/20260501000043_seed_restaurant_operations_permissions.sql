-- Permissions for the restaurant_operations module. Single-colon format
-- respects permissions_code_format. super_admin and store_admin get the full
-- set; cashier/store_manager will get a tighter subset later.
--
-- IMPORTANT: this is a backup. The seed binary
-- (seed/src/data.rs::PERMISSIONS + ROLE_PERMISSIONS) is the actual source of
-- truth for which roles get which perms, since it runs after migrations.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'restaurant:read_station',      'List/read kitchen stations'),
    (gen_random_uuid(), 'restaurant:write_station',     'Create/update/deactivate kitchen stations'),
    (gen_random_uuid(), 'restaurant:read_table',        'List/read restaurant tables'),
    (gen_random_uuid(), 'restaurant:write_table',       'Create/update tables and change their status'),
    (gen_random_uuid(), 'restaurant:read_modifier',     'List/read menu modifier groups + modifiers'),
    (gen_random_uuid(), 'restaurant:write_modifier',    'Create/update modifier groups, modifiers, product M2M'),
    (gen_random_uuid(), 'restaurant:read_ticket',       'List/read KDS tickets and subscribe to the SSE stream'),
    (gen_random_uuid(), 'restaurant:write_ticket',      'Create KDS tickets directly (v1.1 will auto-create from sales)'),
    (gen_random_uuid(), 'restaurant:transition_ticket', 'Send/ready/serve a ticket or change item statuses'),
    (gen_random_uuid(), 'restaurant:cancel_ticket',     'Cancel a KDS ticket')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code LIKE 'restaurant:%'
ON CONFLICT DO NOTHING;
