-- Catalog permissions.
--
-- catalog:read is granted broadly (including to customers).
-- catalog:moderate is for staff approving reviews.
-- catalog:review is for customers submitting reviews.
-- image_storage_providers:* CUD routes are super_admin only at the handler.

INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'catalog:read',     'Read product listings, images, approved reviews, wishlist'),
    (gen_random_uuid(), 'catalog:create',   'Create product listings (manager+)'),
    (gen_random_uuid(), 'catalog:update',   'Update listings, manage images, publish/unpublish'),
    (gen_random_uuid(), 'catalog:delete',   'Delete listings'),
    (gen_random_uuid(), 'catalog:review',   'Submit product reviews (customer)'),
    (gen_random_uuid(), 'catalog:moderate', 'Approve / delete reviews (manager+)'),
    (gen_random_uuid(), 'image_storage_providers:read',   'Read image storage configuration'),
    (gen_random_uuid(), 'image_storage_providers:create', 'Create image storage provider (super_admin)'),
    (gen_random_uuid(), 'image_storage_providers:update', 'Update image storage provider (super_admin)'),
    (gen_random_uuid(), 'image_storage_providers:delete', 'Delete image storage provider (super_admin)')
ON CONFLICT (code) DO NOTHING;
