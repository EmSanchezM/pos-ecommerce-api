-- Restaurant operations: individual modifiers within a group (e.g. "Sin
-- cebolla", "Extra queso", "Término medio"). `price_delta` may be 0, +N or
-- -N (rare, but supported for "remove" discounts).

CREATE TABLE IF NOT EXISTS menu_modifiers (
    id           UUID PRIMARY KEY,
    group_id     UUID NOT NULL REFERENCES menu_modifier_groups(id) ON DELETE CASCADE,
    name         VARCHAR(120) NOT NULL,
    price_delta  NUMERIC(20, 4) NOT NULL DEFAULT 0,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    is_active    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_menu_modifiers_group_active
    ON menu_modifiers (group_id, sort_order)
    WHERE is_active = TRUE;
