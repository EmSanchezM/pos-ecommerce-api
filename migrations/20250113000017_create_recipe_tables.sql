-- Migration: Create recipe tables
-- Bill of Materials (BOM) for composite products with ingredients and substitutes

-- Product Recipes table
CREATE TABLE IF NOT EXISTS product_recipes (
    id UUID PRIMARY KEY,
    product_id UUID NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    yield_quantity NUMERIC(20, 4) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    preparation_time_minutes INTEGER NULL,
    calculate_cost_from_ingredients BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- XOR constraint: exactly one of product_id or variant_id must be set
    CONSTRAINT product_recipes_product_variant_xor CHECK (
        (product_id IS NOT NULL AND variant_id IS NULL) OR
        (product_id IS NULL AND variant_id IS NOT NULL)
    ),

    -- Yield quantity must be positive
    CONSTRAINT product_recipes_yield_check CHECK (yield_quantity > 0)
);

-- Index for product lookups
CREATE INDEX IF NOT EXISTS idx_product_recipes_product_id ON product_recipes(product_id) WHERE product_id IS NOT NULL;

-- Index for variant lookups
CREATE INDEX IF NOT EXISTS idx_product_recipes_variant_id ON product_recipes(variant_id) WHERE variant_id IS NOT NULL;

-- Index for active recipes
CREATE INDEX IF NOT EXISTS idx_product_recipes_is_active ON product_recipes(is_active);

-- Partial unique index: only one active recipe per product
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_recipes_active_product ON product_recipes(product_id)
    WHERE product_id IS NOT NULL AND is_active = TRUE;

-- Partial unique index: only one active recipe per variant
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_recipes_active_variant ON product_recipes(variant_id)
    WHERE variant_id IS NOT NULL AND is_active = TRUE;


-- Recipe Ingredients table
CREATE TABLE IF NOT EXISTS recipe_ingredients (
    id UUID PRIMARY KEY,
    recipe_id UUID NOT NULL REFERENCES product_recipes(id) ON DELETE CASCADE,
    ingredient_product_id UUID NULL REFERENCES products(id) ON DELETE RESTRICT,
    ingredient_variant_id UUID NULL REFERENCES product_variants(id) ON DELETE RESTRICT,
    quantity NUMERIC(20, 4) NOT NULL,
    unit_of_measure VARCHAR(20) NOT NULL,
    is_optional BOOLEAN NOT NULL DEFAULT FALSE,
    can_substitute BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    preparation_step TEXT NULL,
    estimated_cost_per_unit NUMERIC(20, 4) NULL,
    estimated_waste_percentage NUMERIC(5, 4) NOT NULL DEFAULT 0,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- XOR constraint: exactly one of ingredient_product_id or ingredient_variant_id must be set
    CONSTRAINT recipe_ingredients_product_variant_xor CHECK (
        (ingredient_product_id IS NOT NULL AND ingredient_variant_id IS NULL) OR
        (ingredient_product_id IS NULL AND ingredient_variant_id IS NOT NULL)
    ),

    -- Quantity must be positive
    CONSTRAINT recipe_ingredients_quantity_check CHECK (quantity > 0),

    -- Waste percentage must be between 0 and 1
    CONSTRAINT recipe_ingredients_waste_check CHECK (estimated_waste_percentage >= 0 AND estimated_waste_percentage <= 1)
);

-- Index for recipe lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_recipe_id ON recipe_ingredients(recipe_id);

-- Index for ingredient product lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_product_id ON recipe_ingredients(ingredient_product_id)
    WHERE ingredient_product_id IS NOT NULL;

-- Index for ingredient variant lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_variant_id ON recipe_ingredients(ingredient_variant_id)
    WHERE ingredient_variant_id IS NOT NULL;

-- Index for sort order
CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_sort_order ON recipe_ingredients(recipe_id, sort_order);


-- Recipe Ingredient Substitutes table
CREATE TABLE IF NOT EXISTS recipe_ingredient_substitutes (
    id UUID PRIMARY KEY,
    recipe_ingredient_id UUID NOT NULL REFERENCES recipe_ingredients(id) ON DELETE CASCADE,
    substitute_product_id UUID NULL REFERENCES products(id) ON DELETE CASCADE,
    substitute_variant_id UUID NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    conversion_ratio NUMERIC(10, 4) NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- XOR constraint: exactly one of substitute_product_id or substitute_variant_id must be set
    CONSTRAINT recipe_ingredient_substitutes_product_variant_xor CHECK (
        (substitute_product_id IS NOT NULL AND substitute_variant_id IS NULL) OR
        (substitute_product_id IS NULL AND substitute_variant_id IS NOT NULL)
    ),

    -- Conversion ratio must be positive
    CONSTRAINT recipe_ingredient_substitutes_ratio_check CHECK (conversion_ratio > 0),

    -- Priority must be non-negative
    CONSTRAINT recipe_ingredient_substitutes_priority_check CHECK (priority >= 0)
);

-- Index for ingredient lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredient_substitutes_ingredient_id ON recipe_ingredient_substitutes(recipe_ingredient_id);

-- Index for priority ordering
CREATE INDEX IF NOT EXISTS idx_recipe_ingredient_substitutes_priority ON recipe_ingredient_substitutes(recipe_ingredient_id, priority);

-- Index for substitute product lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredient_substitutes_product_id ON recipe_ingredient_substitutes(substitute_product_id)
    WHERE substitute_product_id IS NOT NULL;

-- Index for substitute variant lookups
CREATE INDEX IF NOT EXISTS idx_recipe_ingredient_substitutes_variant_id ON recipe_ingredient_substitutes(substitute_variant_id)
    WHERE substitute_variant_id IS NOT NULL;
