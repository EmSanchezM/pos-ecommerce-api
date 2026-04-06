-- Add purchase_orders:update permission for Phase 2 functionality

INSERT INTO permissions (id, code, description)
VALUES (gen_random_uuid(), 'purchase_orders:update', 'Update draft purchase orders')
ON CONFLICT (code) DO NOTHING;

-- Assign to super_admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'super_admin'
  AND p.code = 'purchase_orders:update'
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );
