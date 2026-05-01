-- Notifications: queued / sent / delivered / failed messages across channels
-- (email, sms, whatsapp, push, webhook). The `metadata` column carries
-- channel-specific extras (e.g. SendGrid template id, FCM topic, headers).
-- See modules/notifications.

CREATE TABLE IF NOT EXISTS notifications (
    id          UUID PRIMARY KEY,
    channel     VARCHAR(20) NOT NULL,
    recipient   VARCHAR(320) NOT NULL,           -- email max length is 320
    subject     TEXT,
    body        TEXT NOT NULL,
    metadata    JSONB NOT NULL DEFAULT '{}'::jsonb,
    status      VARCHAR(20) NOT NULL DEFAULT 'queued',
    attempts    INTEGER NOT NULL DEFAULT 0,
    last_error  TEXT,
    sent_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_status
    ON notifications (status);

-- Hot path for the retry worker: pull failed rows ordered by updated_at.
CREATE INDEX IF NOT EXISTS idx_notifications_retry
    ON notifications (status, attempts, updated_at)
    WHERE status = 'failed';

CREATE INDEX IF NOT EXISTS idx_notifications_recipient
    ON notifications (recipient);

CREATE INDEX IF NOT EXISTS idx_notifications_created_at
    ON notifications (created_at);
