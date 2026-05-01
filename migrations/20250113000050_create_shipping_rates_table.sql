-- Shipping rates: matrix of (method × zone) → tariff.
--
-- rate_type:
--   flat          - charge `base_rate` regardless of weight/order
--   weight_based  - `base_rate + per_kg_rate * weight`
--   order_based   - `base_rate` capped/free above `free_shipping_threshold`
--
-- Optional time-of-day / day-of-week limits (e.g. Hugo only operates 8am-10pm,
-- no Sundays). Null means "always available".

CREATE TABLE IF NOT EXISTS shipping_rates (
    id UUID PRIMARY KEY,
    shipping_method_id UUID NOT NULL REFERENCES shipping_methods(id) ON DELETE CASCADE,
    shipping_zone_id UUID NOT NULL REFERENCES shipping_zones(id) ON DELETE CASCADE,
    rate_type VARCHAR(20) NOT NULL,
    base_rate DECIMAL(15,4) NOT NULL,
    per_kg_rate DECIMAL(15,4) NOT NULL DEFAULT 0,
    free_shipping_threshold DECIMAL(15,4),
    min_order_amount DECIMAL(15,4),
    max_weight_kg DECIMAL(10,2),
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    available_days SMALLINT[] DEFAULT NULL,        -- 0=Sun .. 6=Sat; NULL = all
    available_hour_start SMALLINT,                  -- 0..23, NULL = all day
    available_hour_end SMALLINT,                    -- 0..23
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shipping_rates_method ON shipping_rates(shipping_method_id);
CREATE INDEX IF NOT EXISTS idx_shipping_rates_zone ON shipping_rates(shipping_zone_id);
CREATE INDEX IF NOT EXISTS idx_shipping_rates_active ON shipping_rates(is_active);
