-- Permissions for the demand_planning module: forecast/policy/suggestion/abc.
-- The `permissions.code` constraint enforces a single-colon `module:action`
-- format, so multi-segment actions like "approve_suggestion" are baked into
-- the action half (snake_case allowed).
--
-- All are normal store-level operations; the seed grants them to super_admin
-- and store_admin so existing seeded roles can use the new endpoints.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'demand_planning:read_forecast',     'Read demand forecasts'),
    (gen_random_uuid(), 'demand_planning:read_policy',       'Read reorder policies'),
    (gen_random_uuid(), 'demand_planning:write_policy',      'Create or update reorder policies'),
    (gen_random_uuid(), 'demand_planning:read_suggestion',   'List replenishment suggestions'),
    (gen_random_uuid(), 'demand_planning:approve_suggestion','Approve a replenishment suggestion (creates a Purchase Order)'),
    (gen_random_uuid(), 'demand_planning:dismiss_suggestion','Dismiss a replenishment suggestion'),
    (gen_random_uuid(), 'demand_planning:read_abc',          'Read ABC classification of products')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code IN (
      'demand_planning:read_forecast',
      'demand_planning:read_policy',
      'demand_planning:write_policy',
      'demand_planning:read_suggestion',
      'demand_planning:approve_suggestion',
      'demand_planning:dismiss_suggestion',
      'demand_planning:read_abc'
  )
ON CONFLICT DO NOTHING;
