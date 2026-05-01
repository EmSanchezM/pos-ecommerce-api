-- Transactional outbox: domain events written in the same transaction as the
-- aggregate change. The dispatcher worker drains rows where status='pending'
-- and fans them out to in-process subscribers (analytics, notifications,
-- accounting). See modules/events.

CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id   VARCHAR(128) NOT NULL,
    event_type     VARCHAR(128) NOT NULL,
    payload        JSONB NOT NULL,
    status         VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempts       INTEGER NOT NULL DEFAULT 0,
    last_error     TEXT,
    occurred_at    TIMESTAMPTZ NOT NULL,
    processed_at   TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hot path: dispatcher fetches the next pending events ordered by occurrence.
CREATE INDEX IF NOT EXISTS idx_outbox_events_pending
    ON outbox_events (status, occurred_at)
    WHERE status = 'pending';

-- Lookups by aggregate (debugging, replay, audit).
CREATE INDEX IF NOT EXISTS idx_outbox_events_aggregate
    ON outbox_events (aggregate_type, aggregate_id);

-- Filter by event type (per-subscriber backfills).
CREATE INDEX IF NOT EXISTS idx_outbox_events_event_type
    ON outbox_events (event_type);
