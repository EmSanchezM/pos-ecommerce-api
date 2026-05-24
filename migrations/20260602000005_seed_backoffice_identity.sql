-- Seed backoffice roles and permissions.
--
-- UUIDs are deterministic constants chosen once and frozen.
-- Naming convention: IDs in the b0cf0001-... range are backoffice system seeds.
-- Do NOT change these UUIDs after initial deployment — they are FK targets in
-- backoffice_role_permissions and potentially referenced by tooling/scripts.

-- Roles
INSERT INTO backoffice_roles (id, name, description, is_system_protected, created_at) VALUES
    ('b0cf0001-0000-7000-8000-000000000001', 'super_admin',       'Full platform access — all 13 permissions', TRUE,  NOW()),
    ('b0cf0001-0000-7000-8000-000000000002', 'billing_admin',     'Manage plans, subscriptions and billing',   FALSE, NOW()),
    ('b0cf0001-0000-7000-8000-000000000003', 'support_readonly',  'Read-only access to orgs and audit log',    FALSE, NOW())
ON CONFLICT (id) DO NOTHING;

-- 13 platform permissions (OQ-7 resolution from design)
INSERT INTO backoffice_permissions (id, code, description, created_at) VALUES
    ('b0cf0002-0000-7000-8000-000000000001', 'platform:org.list',                    'List all organizations',              NOW()),
    ('b0cf0002-0000-7000-8000-000000000002', 'platform:org.suspend',                 'Suspend an organization',             NOW()),
    ('b0cf0002-0000-7000-8000-000000000003', 'platform:org.create',                  'Create a new organization',           NOW()),
    ('b0cf0002-0000-7000-8000-000000000004', 'platform:org.update',                  'Update an organization',              NOW()),
    ('b0cf0002-0000-7000-8000-000000000005', 'platform:plan.create',                 'Create a plan',                       NOW()),
    ('b0cf0002-0000-7000-8000-000000000006', 'platform:plan.update',                 'Update a plan',                       NOW()),
    ('b0cf0002-0000-7000-8000-000000000007', 'platform:plan.read',                   'Read plan details',                   NOW()),
    ('b0cf0002-0000-7000-8000-000000000008', 'platform:subscription.force_cancel',   'Force-cancel a subscription',         NOW()),
    ('b0cf0002-0000-7000-8000-000000000009', 'platform:subscription.override_billing','Override subscription billing',      NOW()),
    ('b0cf0002-0000-7000-8000-000000000010', 'platform:dunning.trigger',              'Manually trigger a dunning attempt',  NOW()),
    ('b0cf0002-0000-7000-8000-000000000011', 'platform:audit.read',                  'Read the backoffice audit log',       NOW()),
    ('b0cf0002-0000-7000-8000-000000000012', 'platform:user.impersonate',             'Impersonate a tenant user',           NOW()),
    ('b0cf0002-0000-7000-8000-000000000013', 'platform:analytics.read',               'Read cross-org analytics',           NOW())
ON CONFLICT (id) DO NOTHING;

-- Grant all 13 permissions to super_admin
INSERT INTO backoffice_role_permissions (role_id, permission_id)
SELECT 'b0cf0001-0000-7000-8000-000000000001', id
FROM backoffice_permissions
WHERE code IN (
    'platform:org.list',
    'platform:org.suspend',
    'platform:org.create',
    'platform:org.update',
    'platform:plan.create',
    'platform:plan.update',
    'platform:plan.read',
    'platform:subscription.force_cancel',
    'platform:subscription.override_billing',
    'platform:dunning.trigger',
    'platform:audit.read',
    'platform:user.impersonate',
    'platform:analytics.read'
)
ON CONFLICT DO NOTHING;

-- Grant billing_admin: plan + subscription + dunning permissions
INSERT INTO backoffice_role_permissions (role_id, permission_id)
SELECT 'b0cf0001-0000-7000-8000-000000000002', id
FROM backoffice_permissions
WHERE code IN (
    'platform:plan.create',
    'platform:plan.update',
    'platform:plan.read',
    'platform:subscription.force_cancel',
    'platform:subscription.override_billing',
    'platform:dunning.trigger'
)
ON CONFLICT DO NOTHING;

-- Grant support_readonly: org list + audit read
INSERT INTO backoffice_role_permissions (role_id, permission_id)
SELECT 'b0cf0001-0000-7000-8000-000000000003', id
FROM backoffice_permissions
WHERE code IN (
    'platform:org.list',
    'platform:audit.read'
)
ON CONFLICT DO NOTHING;
