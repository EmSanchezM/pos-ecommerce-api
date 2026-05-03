-- Permissions for the loyalty module. Single-colon format respects
-- permissions_code_format. super_admin and store_admin get the full set;
-- redemption-related perms are granted to cashier later via store_manager
-- when the storefront integration ships in v1.1.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'loyalty:read_program',    'Read loyalty programs'),
    (gen_random_uuid(), 'loyalty:write_program',   'Create or update loyalty programs'),
    (gen_random_uuid(), 'loyalty:read_tier',       'List program tiers'),
    (gen_random_uuid(), 'loyalty:write_tier',      'Create/update program tiers'),
    (gen_random_uuid(), 'loyalty:read_member',     'Read loyalty members + ledgers'),
    (gen_random_uuid(), 'loyalty:enroll_member',   'Enroll a customer into a program'),
    (gen_random_uuid(), 'loyalty:adjust_points',   'Manually adjust a member''s points (audit boundary)'),
    (gen_random_uuid(), 'loyalty:read_reward',     'List rewards'),
    (gen_random_uuid(), 'loyalty:write_reward',    'Create/update rewards'),
    (gen_random_uuid(), 'loyalty:redeem_reward',   'Redeem a member''s points for a reward')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code LIKE 'loyalty:%'
ON CONFLICT DO NOTHING;
