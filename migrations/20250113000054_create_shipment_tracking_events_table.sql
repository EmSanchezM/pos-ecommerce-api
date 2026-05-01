-- Immutable log of shipment status changes / location updates.
-- Source values: system, driver, provider, webhook, manual.

CREATE TABLE IF NOT EXISTS shipment_tracking_events (
    id UUID PRIMARY KEY,
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    status VARCHAR(30) NOT NULL,
    notes TEXT,
    location_lat DECIMAL(10, 7),
    location_lng DECIMAL(10, 7),
    source VARCHAR(20) NOT NULL DEFAULT 'system',
    actor_user_id UUID REFERENCES users(id),
    raw_payload TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tracking_events_shipment
    ON shipment_tracking_events(shipment_id, occurred_at DESC);
