-- Subscriptions: SaaS billing of the platform itself.
--
-- Per the v1.0 plan in docs/roadmap-modulos.md ("Plan detallado — Módulo
-- subscriptions"), this single migration sets up the four tables needed
-- for monthly recurring billing of `Organization`s:
--
--   subscription_plans  — catalog of price + cadence + tier offerings.
--   subscriptions       — one active row per organization (partial unique).
--   billing_cycles      — one row per period charged to a subscription.
--   dunning_attempts    — retry attempts against failed billing cycles.
--
-- Status / interval / outcome strings are CHECK-constrained so a typo at
-- the application layer fails close to the source instead of leaking a
-- bogus value into the table. Soft delete on plans (`is_active = FALSE`).
--
-- See modules/subscriptions.

-- ---------------------------------------------------------------------------
-- subscription_plans
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS subscription_plans (
    id           UUID         PRIMARY KEY,
    code         VARCHAR(32)  NOT NULL UNIQUE,
    name         VARCHAR(160) NOT NULL,
    description  TEXT         NULL,
    tier         VARCHAR(16)  NOT NULL,
    interval     VARCHAR(16)  NOT NULL,
    price_cents  BIGINT       NOT NULL,
    currency     CHAR(3)      NOT NULL,
    trial_days   INTEGER      NULL,
    is_active    BOOLEAN      NOT NULL DEFAULT TRUE,
    sort_order   INTEGER      NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT subscription_plans_code_chk
        CHECK (code ~ '^[a-z0-9_]{3,32}$'),
    CONSTRAINT subscription_plans_tier_chk
        CHECK (tier IN ('free', 'pro', 'enterprise')),
    CONSTRAINT subscription_plans_interval_chk
        CHECK (interval IN ('monthly')),
    CONSTRAINT subscription_plans_price_chk
        CHECK (price_cents >= 0),
    CONSTRAINT subscription_plans_trial_chk
        CHECK (trial_days IS NULL OR trial_days > 0)
);

CREATE INDEX IF NOT EXISTS idx_subscription_plans_active_sort
    ON subscription_plans (sort_order, created_at)
    WHERE is_active = TRUE;

-- ---------------------------------------------------------------------------
-- subscriptions
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS subscriptions (
    id                    UUID        PRIMARY KEY,
    organization_id       UUID        NOT NULL
                                       REFERENCES organizations(id)
                                       ON DELETE RESTRICT,
    plan_id               UUID        NOT NULL
                                       REFERENCES subscription_plans(id)
                                       ON DELETE RESTRICT,
    status                VARCHAR(16) NOT NULL,
    current_period_start  TIMESTAMPTZ NOT NULL,
    current_period_end    TIMESTAMPTZ NOT NULL,
    trial_end             TIMESTAMPTZ NULL,
    cancel_at_period_end  BOOLEAN     NOT NULL DEFAULT FALSE,
    canceled_at           TIMESTAMPTZ NULL,
    version               INTEGER     NOT NULL DEFAULT 1,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT subscriptions_status_chk
        CHECK (status IN ('trialing', 'active', 'past_due', 'canceled')),
    CONSTRAINT subscriptions_period_chk
        CHECK (current_period_end > current_period_start)
);

-- One non-canceled subscription per organization.
CREATE UNIQUE INDEX IF NOT EXISTS uq_subscriptions_org_active
    ON subscriptions (organization_id)
    WHERE status <> 'canceled';

CREATE INDEX IF NOT EXISTS idx_subscriptions_organization
    ON subscriptions (organization_id);

CREATE INDEX IF NOT EXISTS idx_subscriptions_plan
    ON subscriptions (plan_id);

-- Job lookup: subs whose period has elapsed and that are still billable.
CREATE INDEX IF NOT EXISTS idx_subscriptions_due_for_billing
    ON subscriptions (current_period_end)
    WHERE status IN ('trialing', 'active');

-- Job lookup: past_due subs aging out into cancellation.
CREATE INDEX IF NOT EXISTS idx_subscriptions_past_due_updated
    ON subscriptions (updated_at)
    WHERE status = 'past_due';

-- ---------------------------------------------------------------------------
-- billing_cycles
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS billing_cycles (
    id               UUID        PRIMARY KEY,
    subscription_id  UUID        NOT NULL
                                  REFERENCES subscriptions(id)
                                  ON DELETE CASCADE,
    period_start     TIMESTAMPTZ NOT NULL,
    period_end       TIMESTAMPTZ NOT NULL,
    status           VARCHAR(16) NOT NULL,
    invoice_id       UUID        NULL,
    transaction_id   UUID        NULL,
    amount_cents     BIGINT      NOT NULL,
    currency         CHAR(3)     NOT NULL,
    attempted_at     TIMESTAMPTZ NULL,
    settled_at       TIMESTAMPTZ NULL,
    failure_reason   TEXT        NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT billing_cycles_status_chk
        CHECK (status IN ('pending', 'trialing', 'invoiced', 'paid', 'failed', 'skipped')),
    CONSTRAINT billing_cycles_period_chk
        CHECK (period_end > period_start),
    CONSTRAINT billing_cycles_amount_chk
        CHECK (amount_cents >= 0)
);

CREATE INDEX IF NOT EXISTS idx_billing_cycles_subscription
    ON billing_cycles (subscription_id, period_end DESC);

-- Job lookup: cycles ready to be invoiced.
CREATE INDEX IF NOT EXISTS idx_billing_cycles_pending_due
    ON billing_cycles (period_start)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_billing_cycles_transaction
    ON billing_cycles (transaction_id)
    WHERE transaction_id IS NOT NULL;

-- ---------------------------------------------------------------------------
-- dunning_attempts
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS dunning_attempts (
    id                UUID        PRIMARY KEY,
    billing_cycle_id  UUID        NOT NULL
                                   REFERENCES billing_cycles(id)
                                   ON DELETE CASCADE,
    attempt_number    SMALLINT    NOT NULL,
    scheduled_at      TIMESTAMPTZ NOT NULL,
    executed_at       TIMESTAMPTZ NULL,
    outcome           VARCHAR(16) NOT NULL DEFAULT 'pending',
    failure_reason    TEXT        NULL,
    transaction_id    UUID        NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT dunning_attempts_outcome_chk
        CHECK (outcome IN ('pending', 'succeeded', 'failed', 'skipped')),
    CONSTRAINT dunning_attempts_number_chk
        CHECK (attempt_number BETWEEN 1 AND 10),
    CONSTRAINT dunning_attempts_unique_number
        UNIQUE (billing_cycle_id, attempt_number)
);

CREATE INDEX IF NOT EXISTS idx_dunning_attempts_cycle
    ON dunning_attempts (billing_cycle_id, attempt_number);

-- Job lookup: attempts ready to fire.
CREATE INDEX IF NOT EXISTS idx_dunning_attempts_due
    ON dunning_attempts (scheduled_at)
    WHERE outcome = 'pending';
