-- Shipments: one per sale (UNIQUE(sale_id)).
--
-- Polymorphic on shipping_method.method_type — driver_id is set only for
-- own_delivery, delivery_provider_id only for external_delivery, pickup_*
-- fields only for store_pickup. The application enforces these.
--
-- COD bridging:
--   requires_cash_collection = true means there is a payments::Transaction in
--   `pending` for this sale that should be confirmed when status flips to
--   `delivered`. The shipping use case looks up the transaction by sale_id and
--   calls Transaction.confirm() — see MarkDeliveredUseCase.

CREATE TABLE IF NOT EXISTS shipments (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    sale_id UUID NOT NULL REFERENCES sales(id),
    shipping_method_id UUID NOT NULL REFERENCES shipping_methods(id),

    -- Polymorphic per method_type
    driver_id UUID REFERENCES drivers(id),
    delivery_provider_id UUID REFERENCES delivery_providers(id),

    -- StorePickup fields
    pickup_code VARCHAR(20),
    pickup_ready_at TIMESTAMPTZ,
    pickup_expires_at TIMESTAMPTZ,
    picked_up_at TIMESTAMPTZ,
    picked_up_by_name VARCHAR(150),

    -- COD integration with payments
    requires_cash_collection BOOLEAN NOT NULL DEFAULT false,
    cash_amount DECIMAL(15,4),

    -- Common fields
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    tracking_number VARCHAR(100),
    carrier_name VARCHAR(100),
    shipping_cost DECIMAL(15,4) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'HNL',
    weight_kg DECIMAL(10,2),

    recipient_name VARCHAR(255) NOT NULL,
    recipient_phone VARCHAR(30),
    address_line1 VARCHAR(255) NOT NULL,
    address_line2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100) NOT NULL,
    postal_code VARCHAR(20),
    country VARCHAR(3) NOT NULL DEFAULT 'HN',

    notes TEXT,
    failure_reason TEXT,
    attempt_count INT NOT NULL DEFAULT 0,

    shipped_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    estimated_delivery TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancel_reason TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT shipments_sale_unique UNIQUE (sale_id)
);

CREATE INDEX IF NOT EXISTS idx_shipments_store_id ON shipments(store_id);
CREATE INDEX IF NOT EXISTS idx_shipments_sale_id ON shipments(sale_id);
CREATE INDEX IF NOT EXISTS idx_shipments_status ON shipments(status);
CREATE INDEX IF NOT EXISTS idx_shipments_tracking ON shipments(tracking_number);
CREATE INDEX IF NOT EXISTS idx_shipments_driver_id
    ON shipments(driver_id)
    WHERE driver_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_shipments_provider_id
    ON shipments(delivery_provider_id)
    WHERE delivery_provider_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_shipments_pickup_expiring
    ON shipments(pickup_expires_at)
    WHERE pickup_expires_at IS NOT NULL AND status = 'ready_for_pickup';
