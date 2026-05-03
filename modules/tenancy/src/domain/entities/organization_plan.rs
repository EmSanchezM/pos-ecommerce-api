//! OrganizationPlan — one row per organization (UNIQUE on `organization_id`).
//! `feature_flags` is a flat JSON object: `{ "booking": true, "restaurant":
//! false, ... }`. The application layer reads/writes it; v1.1 will gate the
//! relevant routes via a `RequireFeature` middleware.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::TenancyError;
use crate::domain::value_objects::{OrganizationId, OrganizationPlanId, PlanTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPlan {
    id: OrganizationPlanId,
    organization_id: OrganizationId,
    tier: PlanTier,
    feature_flags: JsonValue,
    seat_limit: Option<i32>,
    store_limit: Option<i32>,
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OrganizationPlan {
    pub fn new(
        organization_id: OrganizationId,
        tier: PlanTier,
        feature_flags: Option<JsonValue>,
        seat_limit: Option<i32>,
        store_limit: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, TenancyError> {
        if let Some(seat) = seat_limit
            && seat <= 0
        {
            return Err(TenancyError::Validation(
                "seat_limit must be > 0 if provided".to_string(),
            ));
        }
        if let Some(store) = store_limit
            && store <= 0
        {
            return Err(TenancyError::Validation(
                "store_limit must be > 0 if provided".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: OrganizationPlanId::new(),
            organization_id,
            tier,
            feature_flags: feature_flags.unwrap_or_else(|| tier.default_feature_flags()),
            seat_limit,
            store_limit,
            starts_at: now,
            expires_at,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: OrganizationPlanId,
        organization_id: OrganizationId,
        tier: PlanTier,
        feature_flags: JsonValue,
        seat_limit: Option<i32>,
        store_limit: Option<i32>,
        starts_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            tier,
            feature_flags,
            seat_limit,
            store_limit,
            starts_at,
            expires_at,
            created_at,
            updated_at,
        }
    }

    pub fn update(
        &mut self,
        tier: PlanTier,
        feature_flags: JsonValue,
        seat_limit: Option<i32>,
        store_limit: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), TenancyError> {
        if let Some(seat) = seat_limit
            && seat <= 0
        {
            return Err(TenancyError::Validation(
                "seat_limit must be > 0 if provided".to_string(),
            ));
        }
        if let Some(store) = store_limit
            && store <= 0
        {
            return Err(TenancyError::Validation(
                "store_limit must be > 0 if provided".to_string(),
            ));
        }
        self.tier = tier;
        self.feature_flags = feature_flags;
        self.seat_limit = seat_limit;
        self.store_limit = store_limit;
        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Toggle a single feature flag (used by the `set_feature_flag` use case).
    pub fn set_feature(&mut self, feature: &str, enabled: bool) {
        if !self.feature_flags.is_object() {
            self.feature_flags = serde_json::json!({});
        }
        if let Some(obj) = self.feature_flags.as_object_mut() {
            obj.insert(feature.to_string(), serde_json::Value::Bool(enabled));
        }
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> OrganizationPlanId {
        self.id
    }
    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }
    pub fn tier(&self) -> PlanTier {
        self.tier
    }
    pub fn feature_flags(&self) -> &JsonValue {
        &self.feature_flags
    }
    pub fn seat_limit(&self) -> Option<i32> {
        self.seat_limit
    }
    pub fn store_limit(&self) -> Option<i32> {
        self.store_limit
    }
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
