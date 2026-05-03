-- Restaurant operations: menu modifier groups (e.g. "Cocción", "Extras").
--
-- min_select / max_select drive UI validation. Required groups have
-- min_select >= 1; multi-select extras have max_select > 1.

CREATE TABLE IF NOT EXISTS menu_modifier_groups (
    id         UUID PRIMARY KEY,
    store_id   UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    name       VARCHAR(80) NOT NULL,
    min_select INTEGER NOT NULL DEFAULT 0,
    max_select INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT menu_modifier_groups_select_chk CHECK (min_select >= 0 AND max_select >= min_select)
);

CREATE INDEX IF NOT EXISTS idx_menu_modifier_groups_store_active
    ON menu_modifier_groups (store_id, sort_order)
    WHERE is_active = TRUE;
