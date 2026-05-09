//! `SubscriptionPlan` — the **price + cadence** offered to organizations.
//!
//! v1.0 separates this from `tenancy::OrganizationPlan` (which carries the
//! feature flags and tier limits). The two are linked by `tier`: when a
//! subscription activates, a tenancy subscriber syncs the org's plan tier
//! and snaps the feature flags to the tier defaults.
//!
//! Soft delete via `is_active = false`; hard delete is never used.

use chrono::{DateTime, Utc};

use tenancy::PlanTier;

use crate::domain::value_objects::{BillingInterval, PlanCode, SubscriptionPlanId};

#[derive(Debug, Clone)]
pub struct SubscriptionPlan {
    id: SubscriptionPlanId,
    code: PlanCode,
    name: String,
    description: Option<String>,
    tier: PlanTier,
    interval: BillingInterval,
    /// Price in the smallest currency unit (e.g. cents for USD/HNL).
    price_cents: i64,
    /// ISO-4217 currency code (e.g. "USD", "HNL").
    currency: String,
    /// If `Some`, the first billing cycle starts as Trialing and is
    /// auto-skipped at trial end. `None` means "no trial — bill immediately".
    trial_days: Option<i32>,
    is_active: bool,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SubscriptionPlan {
    /// Creates a brand-new plan. The created plan is `is_active = true` and
    /// `sort_order = 0` by default; admins adjust ordering via
    /// `set_sort_order`. `trial_days` defaults to `None`; admins set it via
    /// `set_trial_days` once the plan exists.
    pub fn create(
        code: PlanCode,
        name: String,
        tier: PlanTier,
        interval: BillingInterval,
        price_cents: i64,
        currency: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SubscriptionPlanId::new(),
            code,
            name,
            description: None,
            tier,
            interval,
            price_cents,
            currency,
            trial_days: None,
            is_active: true,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Rebuilds a plan from its persisted form. Repository-only.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SubscriptionPlanId,
        code: PlanCode,
        name: String,
        description: Option<String>,
        tier: PlanTier,
        interval: BillingInterval,
        price_cents: i64,
        currency: String,
        trial_days: Option<i32>,
        is_active: bool,
        sort_order: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            name,
            description,
            tier,
            interval,
            price_cents,
            currency,
            trial_days,
            is_active,
            sort_order,
            created_at,
            updated_at,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn rename(&mut self, name: String, description: Option<String>) {
        self.name = name;
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
    }

    pub fn set_trial_days(&mut self, trial_days: Option<i32>) {
        self.trial_days = trial_days;
        self.updated_at = Utc::now();
    }

    // ---------------------------------------------------------------------
    // Getters
    // ---------------------------------------------------------------------

    pub fn id(&self) -> SubscriptionPlanId {
        self.id
    }
    pub fn code(&self) -> &PlanCode {
        &self.code
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn tier(&self) -> PlanTier {
        self.tier
    }
    pub fn interval(&self) -> BillingInterval {
        self.interval
    }
    pub fn price_cents(&self) -> i64 {
        self.price_cents
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn trial_days(&self) -> Option<i32> {
        self.trial_days
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
