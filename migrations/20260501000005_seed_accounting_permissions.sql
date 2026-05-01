-- Permissions for the accounting module. Both are normal store-level
-- operations (chart of accounts, journal entries, P&L). The seed grants
-- both to super_admin and store_admin via the role permission seeder.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'accounting:read',  'Read chart of accounts, periods, journal entries, and reports'),
    (gen_random_uuid(), 'accounting:write', 'Create accounts, open/close periods, post journal entries')
ON CONFLICT (code) DO NOTHING;

-- Auto-grant to super_admin and store_admin so existing seeded roles can use
-- the new endpoints immediately.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code IN ('accounting:read', 'accounting:write')
ON CONFLICT DO NOTHING;
