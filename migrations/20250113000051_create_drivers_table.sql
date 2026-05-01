-- Drivers: store-owned delivery personnel.
--
-- `user_id` is optional because subcontracted drivers in HN are common and
-- don't always have system accounts. `current_status` is denormalized state
-- to avoid recomputing from in-flight shipments on every dispatch decision.

CREATE TABLE IF NOT EXISTS drivers (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    user_id UUID REFERENCES users(id),
    name VARCHAR(150) NOT NULL,
    phone VARCHAR(30) NOT NULL,
    vehicle_type VARCHAR(20) NOT NULL,         -- motorcycle, car, bicycle, pickup, foot
    license_plate VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    current_status VARCHAR(20) NOT NULL DEFAULT 'offline',  -- offline, available, busy
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT drivers_store_phone_unique UNIQUE (store_id, phone)
);

CREATE INDEX IF NOT EXISTS idx_drivers_store_id ON drivers(store_id);
CREATE INDEX IF NOT EXISTS idx_drivers_active_available
    ON drivers(store_id, current_status)
    WHERE is_active = true AND current_status = 'available';
