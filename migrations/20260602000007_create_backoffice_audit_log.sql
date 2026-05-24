-- Migration: 20260602000007_create_backoffice_audit_log
--
-- Creates the append-only backoffice audit log table.
--
-- Design decisions (FR-AUD-4, NFR-SEC-2, C-3):
-- - NO updated_at column: this table is append-only by design.
-- - NO upsert-friendly unique constraints: each row is a distinct immutable event.
-- - UUID v7 PK provides monotonic ordering without a sequence.
-- - 4 indexes: PK on id, actor_id, partial on target_org_id, occurred_at.
--
-- The `occurred_at` column defaults to NOW() so callers don't need to supply it
-- (the subscriber writes via INSERT without specifying occurred_at).

CREATE TABLE IF NOT EXISTS backoffice_audit_log (
    id              UUID        PRIMARY KEY,
    actor_id        UUID        NOT NULL,
    actor_type      VARCHAR     NOT NULL,
    action          VARCHAR     NOT NULL,
    target_org_id   UUID            NULL,
    reason          TEXT        NOT NULL,
    ip              VARCHAR     NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for actor lookups ("show me all actions by operator X")
CREATE INDEX IF NOT EXISTS idx_backoffice_audit_log_actor_id
    ON backoffice_audit_log (actor_id);

-- Partial index on target_org_id — only indexes rows where it is set,
-- matching the anticipated query "show me all actions affecting org Y".
CREATE INDEX IF NOT EXISTS idx_backoffice_audit_log_target_org_id
    ON backoffice_audit_log (target_org_id)
    WHERE target_org_id IS NOT NULL;

-- Index on occurred_at for efficient time-ordered pagination.
CREATE INDEX IF NOT EXISTS idx_backoffice_audit_log_occurred_at
    ON backoffice_audit_log (occurred_at DESC);
