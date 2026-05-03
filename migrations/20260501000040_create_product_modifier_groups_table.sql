-- Restaurant operations: M2M between menu products and modifier groups
-- eligible to be applied to them. Empty set = the product has no modifiers.

CREATE TABLE IF NOT EXISTS product_modifier_groups (
    product_id UUID NOT NULL REFERENCES products(id)             ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES menu_modifier_groups(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, group_id)
);

CREATE INDEX IF NOT EXISTS idx_product_modifier_groups_group
    ON product_modifier_groups (group_id);
