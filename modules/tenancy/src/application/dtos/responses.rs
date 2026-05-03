use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::entities::{
    Organization, OrganizationBranding, OrganizationDomain, OrganizationPlan,
};
use crate::domain::value_objects::{OrganizationStatus, OrganizationTheme, PlanTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub status: OrganizationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Organization> for OrganizationResponse {
    fn from(o: &Organization) -> Self {
        Self {
            id: o.id().into_uuid(),
            name: o.name().to_string(),
            slug: o.slug().to_string(),
            contact_email: o.contact_email().to_string(),
            contact_phone: o.contact_phone().map(String::from),
            status: o.status(),
            created_at: o.created_at(),
            updated_at: o.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPlanResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tier: PlanTier,
    pub feature_flags: JsonValue,
    pub seat_limit: Option<i32>,
    pub store_limit: Option<i32>,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<&OrganizationPlan> for OrganizationPlanResponse {
    fn from(p: &OrganizationPlan) -> Self {
        Self {
            id: p.id().into_uuid(),
            organization_id: p.organization_id().into_uuid(),
            tier: p.tier(),
            feature_flags: p.feature_flags().clone(),
            seat_limit: p.seat_limit(),
            store_limit: p.store_limit(),
            starts_at: p.starts_at(),
            expires_at: p.expires_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDomainResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain: String,
    pub is_verified: bool,
    pub is_primary: bool,
    /// The verification token is only returned right after registration. Once
    /// the domain is verified, the column is nulled out and the response
    /// reflects that.
    pub verification_token: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<&OrganizationDomain> for OrganizationDomainResponse {
    fn from(d: &OrganizationDomain) -> Self {
        Self {
            id: d.id().into_uuid(),
            organization_id: d.organization_id().into_uuid(),
            domain: d.domain().to_string(),
            is_verified: d.is_verified(),
            is_primary: d.is_primary(),
            verification_token: d.verification_token().map(String::from),
            verified_at: d.verified_at(),
            created_at: d.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationBrandingResponse {
    pub organization_id: Uuid,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub theme: OrganizationTheme,
    pub custom_css: Option<String>,
}

impl From<&OrganizationBranding> for OrganizationBrandingResponse {
    fn from(b: &OrganizationBranding) -> Self {
        Self {
            organization_id: b.organization_id().into_uuid(),
            logo_url: b.logo_url().map(String::from),
            favicon_url: b.favicon_url().map(String::from),
            primary_color: b.primary_color().map(String::from),
            secondary_color: b.secondary_color().map(String::from),
            accent_color: b.accent_color().map(String::from),
            theme: b.theme(),
            custom_css: b.custom_css().map(String::from),
        }
    }
}

/// Bundled detail returned by `GET /organizations/{id}` — saves the dashboard
/// the round-trip cost of fetching plan, domains, and branding separately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDetailResponse {
    #[serde(flatten)]
    pub organization: OrganizationResponse,
    pub plan: Option<OrganizationPlanResponse>,
    pub domains: Vec<OrganizationDomainResponse>,
    pub branding: Option<OrganizationBrandingResponse>,
}

/// Public payload for the storefront — strips contact info, plan, domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicOrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub branding: Option<OrganizationBrandingResponse>,
}
