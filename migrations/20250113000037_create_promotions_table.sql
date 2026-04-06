-- Promotions table for discount codes, percentage/fixed/buy-x-get-y promotions
CREATE TABLE promotions (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    promotion_type VARCHAR(30) NOT NULL,  -- 'percentage', 'fixed_amount', 'buy_x_get_y'
    status VARCHAR(20) NOT NULL DEFAULT 'active',  -- 'active', 'inactive', 'expired'
    discount_value DECIMAL(15,4) NOT NULL DEFAULT 0,
    buy_quantity INT,          -- For buy_x_get_y: buy X items
    get_quantity INT,          -- For buy_x_get_y: get Y items free/discounted
    minimum_purchase DECIMAL(15,4) NOT NULL DEFAULT 0,
    maximum_discount DECIMAL(15,4),  -- Cap on discount amount
    usage_limit INT,           -- Max total uses (NULL = unlimited)
    usage_count INT NOT NULL DEFAULT 0,
    per_customer_limit INT,    -- Max uses per customer (NULL = unlimited)
    applies_to VARCHAR(30) NOT NULL DEFAULT 'order',  -- 'order', 'product', 'category'
    product_ids UUID[],        -- Specific products (when applies_to = 'product')
    category_ids UUID[],       -- Specific categories (when applies_to = 'category')
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,      -- NULL = no end date
    store_id UUID REFERENCES stores(id),  -- NULL = all stores
    created_by_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT promotions_code_unique UNIQUE (code),
    CONSTRAINT promotions_discount_value_positive CHECK (discount_value >= 0),
    CONSTRAINT promotions_minimum_purchase_positive CHECK (minimum_purchase >= 0)
);

CREATE INDEX idx_promotions_code ON promotions(code);
CREATE INDEX idx_promotions_status ON promotions(status);
CREATE INDEX idx_promotions_dates ON promotions(start_date, end_date);
CREATE INDEX idx_promotions_store ON promotions(store_id);

-- Permissions for promotions
INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'promotions:create', 'Create promotions'),
    (gen_random_uuid(), 'promotions:read', 'View promotions'),
    (gen_random_uuid(), 'promotions:update', 'Update promotions'),
    (gen_random_uuid(), 'promotions:apply', 'Apply promotions to sales')
ON CONFLICT (code) DO NOTHING;

-- Assign to super_admin
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'super_admin'
  AND p.code IN ('promotions:create', 'promotions:read', 'promotions:update', 'promotions:apply')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- E-Commerce order permissions
INSERT INTO permissions (id, code, description) VALUES
    (gen_random_uuid(), 'orders:mark_paid', 'Mark order as paid'),
    (gen_random_uuid(), 'orders:process', 'Start processing order'),
    (gen_random_uuid(), 'orders:ship', 'Ship order'),
    (gen_random_uuid(), 'orders:deliver', 'Mark order delivered'),
    (gen_random_uuid(), 'orders:cancel', 'Cancel order')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'super_admin'
  AND p.code IN ('orders:mark_paid', 'orders:process', 'orders:ship', 'orders:deliver', 'orders:cancel')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );
