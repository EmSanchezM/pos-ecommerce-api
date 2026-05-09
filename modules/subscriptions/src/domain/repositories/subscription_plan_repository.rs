use async_trait::async_trait;

use crate::SubscriptionError;
use crate::domain::entities::SubscriptionPlan;
use crate::domain::value_objects::{PlanCode, SubscriptionPlanId};

#[async_trait]
pub trait SubscriptionPlanRepository: Send + Sync {
    /// Insert a new plan.
    async fn save(&self, plan: &SubscriptionPlan) -> Result<(), SubscriptionError>;

    /// Update mutable fields of an existing plan (immutable bits — `code`,
    /// `tier`, `interval`, `price_cents` — are validated by the use case to
    /// be unchanged; this method writes whatever the entity currently holds).
    async fn update(&self, plan: &SubscriptionPlan) -> Result<(), SubscriptionError>;

    async fn find_by_id(
        &self,
        id: SubscriptionPlanId,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError>;

    async fn find_by_code(
        &self,
        code: &PlanCode,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError>;

    /// All `is_active = true` plans, ordered by `(sort_order, created_at)`.
    /// Used by the public `/subscription-plans` listing.
    async fn find_active(&self) -> Result<Vec<SubscriptionPlan>, SubscriptionError>;

    /// Admin paginated listing — returns the page slice plus the total row
    /// count for cursor/page rendering.
    async fn list_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<SubscriptionPlan>, i64), SubscriptionError>;
}
