use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{OrganizationTheme, PlanTier};

// -----------------------------------------------------------------------------
// Organization
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterOrganizationCommand {
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    /// Optional initial plan tier (defaults to Free).
    #[serde(default)]
    pub initial_tier: Option<PlanTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationCommand {
    pub name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
}

// -----------------------------------------------------------------------------
// Plan
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPlanCommand {
    pub tier: PlanTier,
    /// Optional override for the feature flags. If absent the use case keeps
    /// the existing flags (or, on first set, applies the tier's defaults).
    #[serde(default)]
    pub feature_flags: Option<JsonValue>,
    pub seat_limit: Option<i32>,
    pub store_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFeatureFlagCommand {
    pub feature: String,
    pub enabled: bool,
}

// -----------------------------------------------------------------------------
// Domain
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDomainCommand {
    pub domain: String,
}

// -----------------------------------------------------------------------------
// Branding
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBrandingCommand {
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    #[serde(default)]
    pub theme: Option<OrganizationTheme>,
    pub custom_css: Option<String>,
}
