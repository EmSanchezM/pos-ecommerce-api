use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};

use crate::TenancyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanTier {
    #[default]
    Free,
    Pro,
    Enterprise,
}

impl PlanTier {
    pub fn as_str(self) -> &'static str {
        match self {
            PlanTier::Free => "free",
            PlanTier::Pro => "pro",
            PlanTier::Enterprise => "enterprise",
        }
    }

    /// Default feature flag set for the tier. The application layer uses this
    /// when an org's plan is initially provisioned; admins can later toggle
    /// individual flags via the plan endpoints.
    pub fn default_feature_flags(self) -> JsonValue {
        match self {
            PlanTier::Free => json!({
                "booking": false,
                "restaurant": false,
                "service_orders": false,
                "loyalty": false,
            }),
            PlanTier::Pro => json!({
                "booking": true,
                "restaurant": true,
                "service_orders": false,
                "loyalty": true,
            }),
            PlanTier::Enterprise => json!({
                "booking": true,
                "restaurant": true,
                "service_orders": true,
                "loyalty": true,
            }),
        }
    }
}

impl FromStr for PlanTier {
    type Err = TenancyError;
    fn from_str(s: &str) -> Result<Self, TenancyError> {
        match s {
            "free" => Ok(PlanTier::Free),
            "pro" => Ok(PlanTier::Pro),
            "enterprise" => Ok(PlanTier::Enterprise),
            other => Err(TenancyError::InvalidTier(other.to_string())),
        }
    }
}
