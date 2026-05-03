-- Permissions for the booking module. Single-colon format respects
-- permissions_code_format. super_admin and store_admin get the full set;
-- public booking endpoints do not require any permission.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'booking:read_resource',          'List/read booking resources (people, equipment, rooms)'),
    (gen_random_uuid(), 'booking:write_resource',         'Create/update/deactivate booking resources and their calendars'),
    (gen_random_uuid(), 'booking:read_service',           'List/read bookable services'),
    (gen_random_uuid(), 'booking:write_service',          'Create/update/deactivate bookable services and assign resources'),
    (gen_random_uuid(), 'booking:read_appointment',       'List/read appointments'),
    (gen_random_uuid(), 'booking:write_appointment',      'Create appointments on behalf of a customer (walk-in, phone)'),
    (gen_random_uuid(), 'booking:transition_appointment', 'Confirm/start/complete/no-show an appointment'),
    (gen_random_uuid(), 'booking:cancel_appointment',     'Cancel an appointment'),
    (gen_random_uuid(), 'booking:read_policy',            'Read the per-store booking policy'),
    (gen_random_uuid(), 'booking:write_policy',           'Upsert the per-store booking policy')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name IN ('super_admin', 'store_admin')
  AND p.code LIKE 'booking:%'
ON CONFLICT DO NOTHING;
