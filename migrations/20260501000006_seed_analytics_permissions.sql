-- Permissions for the analytics module. Both are normal store-level
-- operations (KPI snapshots, dashboards/widgets, reports). Granted by
-- default to super_admin and store_admin so existing seeded roles can
-- use the new endpoints immediately.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'reports:analytics', 'Read analytics dashboards, KPIs and reports'),
    (gen_random_uuid(), 'analytics:write',   'Create dashboards and add/remove widgets')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code IN ('reports:analytics', 'analytics:write')
ON CONFLICT DO NOTHING;
