use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TenancyError {
    #[error("Organization not found: {0}")]
    OrganizationNotFound(Uuid),

    #[error("Organization not found for slug: {0}")]
    OrganizationNotFoundBySlug(String),

    #[error("Plan not found for organization: {0}")]
    PlanNotFound(Uuid),

    #[error("Domain not found: {0}")]
    DomainNotFound(Uuid),

    #[error("Domain not found for hostname: {0}")]
    DomainNotFoundByHostname(String),

    #[error("Branding not found for organization: {0}")]
    BrandingNotFound(Uuid),

    #[error("Slug already taken: {0}")]
    SlugAlreadyTaken(String),

    #[error("Domain already taken: {0}")]
    DomainAlreadyTaken(String),

    #[error("Invalid status transition: {from} → {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Invalid organization status: {0}")]
    InvalidStatus(String),

    #[error("Invalid plan tier: {0}")]
    InvalidTier(String),

    #[error("Invalid theme: {0}")]
    InvalidTheme(String),

    #[error(
        "Invalid slug `{0}`: must be 3-60 chars, lowercase alphanumeric or hyphen, no leading/trailing hyphen"
    )]
    InvalidSlug(String),

    #[error("Invalid domain `{0}`: must look like a valid hostname")]
    InvalidDomain(String),

    #[error("Invalid color `{0}`: must be #RRGGBB hex")]
    InvalidColor(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
