-- Permissions for the shipping module.
--
-- delivery_providers:* mutating routes are super_admin-gated at the handler
-- layer (mirrors payment_gateways). drivers:* and shipments:* are normal
-- store-level operations.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'shipping:read',             'Read shipping configuration (methods/zones/rates)'),
    (gen_random_uuid(), 'shipping:create',           'Create shipping methods/zones/rates'),
    (gen_random_uuid(), 'shipping:update',           'Update shipping methods/zones/rates'),
    (gen_random_uuid(), 'shipping:delete',           'Delete shipping methods/zones/rates'),
    (gen_random_uuid(), 'shipments:read',            'Read shipments'),
    (gen_random_uuid(), 'shipments:create',          'Create shipments'),
    (gen_random_uuid(), 'shipments:update',          'Update shipment status / tracking'),
    (gen_random_uuid(), 'shipments:assign',          'Assign / reassign drivers'),
    (gen_random_uuid(), 'shipments:cancel',          'Cancel shipments'),
    (gen_random_uuid(), 'drivers:read',              'Read drivers'),
    (gen_random_uuid(), 'drivers:create',            'Create drivers'),
    (gen_random_uuid(), 'drivers:update',            'Update drivers'),
    (gen_random_uuid(), 'drivers:delete',            'Delete drivers'),
    (gen_random_uuid(), 'delivery_providers:read',   'Read delivery provider configuration'),
    (gen_random_uuid(), 'delivery_providers:create', 'Create delivery provider (super_admin only)'),
    (gen_random_uuid(), 'delivery_providers:update', 'Update delivery provider (super_admin only)'),
    (gen_random_uuid(), 'delivery_providers:delete', 'Delete delivery provider (super_admin only)')
ON CONFLICT (code) DO NOTHING;
