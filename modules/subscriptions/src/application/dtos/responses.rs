//! Output DTOs returned by subscription use cases.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{BillingCycle, DunningAttempt, Subscription, SubscriptionPlan};

#[derive(Debug, Clone, Serialize)]
pub struct PlanResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tier: String,
    pub interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubscriptionPlan> for PlanResponse {
    fn from(p: SubscriptionPlan) -> Self {
        Self {
            id: p.id().into_uuid(),
            code: p.code().as_str().to_string(),
            name: p.name().to_string(),
            description: p.description().map(str::to_string),
            tier: p.tier().as_str().to_string(),
            interval: p.interval().as_str().to_string(),
            price_cents: p.price_cents(),
            currency: p.currency().to_string(),
            trial_days: p.trial_days(),
            is_active: p.is_active(),
            sort_order: p.sort_order(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedPlans {
    pub items: Vec<PlanResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Subscription> for SubscriptionResponse {
    fn from(s: Subscription) -> Self {
        Self {
            id: s.id().into_uuid(),
            organization_id: s.organization_id().into_uuid(),
            plan_id: s.plan_id().into_uuid(),
            status: s.status().as_str().to_string(),
            current_period_start: s.current_period_start(),
            current_period_end: s.current_period_end(),
            trial_end: s.trial_end(),
            cancel_at_period_end: s.cancel_at_period_end(),
            canceled_at: s.canceled_at(),
            version: s.version(),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BillingCycleResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: String,
    pub invoice_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub amount_cents: i64,
    pub currency: String,
    pub attempted_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<BillingCycle> for BillingCycleResponse {
    fn from(c: BillingCycle) -> Self {
        Self {
            id: c.id().into_uuid(),
            subscription_id: c.subscription_id().into_uuid(),
            period_start: c.period_start(),
            period_end: c.period_end(),
            status: c.status().as_str().to_string(),
            invoice_id: c.invoice_id(),
            transaction_id: c.transaction_id(),
            amount_cents: c.amount_cents(),
            currency: c.currency().to_string(),
            attempted_at: c.attempted_at(),
            settled_at: c.settled_at(),
            failure_reason: c.failure_reason().map(str::to_string),
            created_at: c.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedBillingCycles {
    pub items: Vec<BillingCycleResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DunningAttemptResponse {
    pub id: Uuid,
    pub billing_cycle_id: Uuid,
    pub attempt_number: i16,
    pub scheduled_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub outcome: String,
    pub failure_reason: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<DunningAttempt> for DunningAttemptResponse {
    fn from(a: DunningAttempt) -> Self {
        Self {
            id: a.id().into_uuid(),
            billing_cycle_id: a.billing_cycle_id().into_uuid(),
            attempt_number: a.attempt_number(),
            scheduled_at: a.scheduled_at(),
            executed_at: a.executed_at(),
            outcome: a.outcome().as_str().to_string(),
            failure_reason: a.failure_reason().map(str::to_string),
            transaction_id: a.transaction_id(),
            created_at: a.created_at(),
        }
    }
}

/// Aggregate result returned by the periodic billing tick. Useful for
/// observability + tests.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BillingTickReport {
    pub trial_activated: i64,
    pub period_advanced: i64,
    pub cycles_invoiced: i64,
    pub dunning_executed: i64,
    pub past_due_canceled: i64,
}
