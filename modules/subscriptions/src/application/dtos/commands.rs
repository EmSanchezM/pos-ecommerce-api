//! Input DTOs for subscription use cases.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlanCommand {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// `"free" | "pro" | "enterprise"`.
    pub tier: String,
    /// `"monthly"` for v1.0.
    pub interval: String,
    pub price_cents: i64,
    /// ISO-4217 (e.g. `"USD"`, `"HNL"`).
    pub currency: String,
    pub trial_days: Option<i32>,
    #[serde(default)]
    pub sort_order: i32,
}

/// Mutable fields on a plan. `code`, `tier`, `interval`, `price_cents`, and
/// `currency` are intentionally **not** updatable — billing already invoiced
/// against a price must stay reproducible. Migration path: deactivate the old
/// plan and create a new one with the new price.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePlanCommand {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub trial_days: Option<Option<i32>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeOrganizationCommand {
    pub organization_id: Uuid,
    pub plan_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelSubscriptionCommand {
    pub organization_id: Uuid,
    /// `false` → cancel at period end (default). `true` → terminate
    /// immediately; restricted to super_admin at the handler layer.
    #[serde(default)]
    pub immediately: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResumeSubscriptionCommand {
    pub organization_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePlanCommand {
    pub organization_id: Uuid,
    pub new_plan_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListBillingCyclesQuery {
    pub organization_id: Uuid,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPlansQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}
