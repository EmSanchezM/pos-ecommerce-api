-- Service order quotes — versioned cost estimates.
--
-- Each new quote for the same order bumps `version` and the application's
-- create-quote use case marks any prior `Draft|Sent` quote as `Superseded`,
-- so at most one active quote exists per order at any time. Older quotes are
-- kept for audit.
--
-- Status workflow:
--   draft → sent → {approved, rejected}
--   draft|sent → superseded (when a newer quote is drafted)

CREATE TABLE IF NOT EXISTS service_quotes (
    id                  UUID PRIMARY KEY,
    service_order_id    UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    version             INTEGER NOT NULL,
    labor_total         NUMERIC(20, 4) NOT NULL DEFAULT 0,
    parts_total         NUMERIC(20, 4) NOT NULL DEFAULT 0,
    tax_total           NUMERIC(20, 4) NOT NULL DEFAULT 0,
    grand_total         NUMERIC(20, 4) NOT NULL DEFAULT 0,
    valid_until         TIMESTAMPTZ NULL,
    notes               TEXT NULL,
    status              VARCHAR(12) NOT NULL DEFAULT 'draft',
    sent_at             TIMESTAMPTZ NULL,
    decided_at          TIMESTAMPTZ NULL,
    decided_by_customer BOOLEAN NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT service_quotes_status_chk
        CHECK (status IN ('draft', 'sent', 'approved', 'rejected', 'superseded')),
    CONSTRAINT service_quotes_version_chk    CHECK (version >= 1),
    CONSTRAINT service_quotes_unique_version UNIQUE (service_order_id, version)
);

CREATE INDEX IF NOT EXISTS idx_service_quotes_order_status
    ON service_quotes (service_order_id, status);
