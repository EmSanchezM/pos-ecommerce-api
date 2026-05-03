-- Service order diagnostics — technician findings.
--
-- Multiple diagnostics per order are allowed (re-evaluations after testing).
-- `severity` drives UI badges and (later) escalation rules.

CREATE TABLE IF NOT EXISTS service_diagnostics (
    id                 UUID PRIMARY KEY,
    service_order_id   UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    technician_user_id UUID NULL     REFERENCES users(id)          ON DELETE SET NULL,
    findings           TEXT NOT NULL,
    recommended_actions TEXT NULL,
    severity           VARCHAR(10) NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT service_diagnostics_severity_chk
        CHECK (severity IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX IF NOT EXISTS idx_service_diagnostics_order
    ON service_diagnostics (service_order_id, created_at DESC);
