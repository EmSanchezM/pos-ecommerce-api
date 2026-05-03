-- Permissions for the tenancy module. v1.0 grants the full set only to
-- super_admin; v1.1 introduces an `org_admin` role with a subset (read_org,
-- write_branding, write_domain — but NOT write_plan).
--
-- IMPORTANT: this is a backup. The seed binary
-- (seed/src/data.rs::PERMISSIONS + ROLE_PERMISSIONS) is the actual source
-- of truth, since it runs after migrations.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'tenancy:read_org',        'List/read organizations'),
    (gen_random_uuid(), 'tenancy:write_org',       'Register or update an organization (name + contact)'),
    (gen_random_uuid(), 'tenancy:suspend_org',     'Suspend or re-activate an organization'),
    (gen_random_uuid(), 'tenancy:read_plan',       'Read an organization plan (tier + feature flags + limits)'),
    (gen_random_uuid(), 'tenancy:write_plan',      'Set the plan tier, toggle feature flags, change limits'),
    (gen_random_uuid(), 'tenancy:read_domain',     'List/read custom domains'),
    (gen_random_uuid(), 'tenancy:write_domain',    'Register, set primary, or delete a custom domain'),
    (gen_random_uuid(), 'tenancy:verify_domain',   'Mark a custom domain as verified'),
    (gen_random_uuid(), 'tenancy:read_branding',   'Read an organization branding (colors, logo, theme)'),
    (gen_random_uuid(), 'tenancy:write_branding',  'Upsert an organization branding')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'super_admin'
  AND p.code LIKE 'tenancy:%'
ON CONFLICT DO NOTHING;
