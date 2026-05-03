//! # Tenancy Module
//!
//! Top-level "platform" entity. Adds the `Organization` (tenant) layer above
//! `store_id` so the API can serve multiple commerce orgs from a single
//! deployment, each with its own custom domains, branding, plan tier, and
//! feature flag set.
//!
//! - **Domain**: `Organization`, `OrganizationPlan` (tier + feature_flags
//!   JSONB + seat/store limits + expires_at), `OrganizationDomain`
//!   (custom hostnames with verification token + partial unique
//!   `is_primary`), `OrganizationBranding` (logo, colors, theme, custom CSS).
//!
//! - **Application**: CRUD use cases for orgs (with suspend/activate),
//!   `SetPlanUseCase` + `SetFeatureFlagUseCase` (plan tier + per-feature
//!   toggle), domain register/verify/set-primary, branding upsert,
//!   public-by-slug / by-domain lookups for the storefront.
//!
//! - **Infrastructure**: 4 `Pg*Repository` SQLx implementations.
//!   `set_primary` runs in a transaction (clear-others + set-target) to
//!   maintain the partial unique index.
//!
//! ## Decisions parked for v1.1+
//!
//! - **Enforcement**: v1.0 only persists the data. v1.1 adds an
//!   `OrganizationScope` extractor (filters every authenticated query by the
//!   user's `organization_id`) plus a `RequireFeature` middleware (returns
//!   403 when a route hits a module that the org's plan disables).
//! - **JWT enrichment**: v1.0 keeps `auth_handlers::login` untouched. v1.1
//!   adds `organization_id` to the JWT claim by reading
//!   `users.organization_id` (with a fallback to the default org).
//! - **DNS verification**: v1.0 lets admins mark domains as verified
//!   manually. v1.1 adds a job that queries DNS TXT and auto-verifies.
//! - **Host-header → org middleware**: v1.1 reads `Host:` and resolves to
//!   `organization_id` from `organization_domains` for the storefront.
//! - **NOT NULL on `users.organization_id` and `stores.organization_id`**:
//!   v1.2 flips both columns to `NOT NULL` after backfill is verified in
//!   prod.
//!
//! See `docs/roadmap-modulos.md` (Fase 3.1 + "Plan detallado — Módulo
//! tenancy") for the full contract.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::TenancyError;

// Domain
pub use domain::entities::{
    Organization, OrganizationBranding, OrganizationDomain, OrganizationPlan,
};
pub use domain::repositories::{
    OrganizationBrandingRepository, OrganizationDomainRepository, OrganizationPlanRepository,
    OrganizationRepository,
};
pub use domain::value_objects::{
    OrganizationDomainId, OrganizationId, OrganizationPlanId, OrganizationStatus,
    OrganizationTheme, PlanTier,
};

// Application
pub use application::dtos::{
    OrganizationBrandingResponse, OrganizationDetailResponse, OrganizationDomainResponse,
    OrganizationPlanResponse, OrganizationResponse, PublicOrganizationResponse,
    RegisterDomainCommand, RegisterOrganizationCommand, SetFeatureFlagCommand, SetPlanCommand,
    UpdateOrganizationCommand, UpsertBrandingCommand,
};
pub use application::subscriber::TenancyEventSubscriber;
pub use application::use_cases::{
    ActivateOrganizationUseCase, DeleteDomainUseCase, FindOrganizationByDomainUseCase,
    GetBrandingUseCase, GetOrganizationBySlugUseCase, GetOrganizationUseCase, GetPlanUseCase,
    ListDomainsUseCase, ListOrganizationsUseCase, RegisterDomainUseCase,
    RegisterOrganizationUseCase, SetFeatureFlagUseCase, SetPlanUseCase, SetPrimaryDomainUseCase,
    SuspendOrganizationUseCase, UpdateOrganizationUseCase, UpsertBrandingUseCase,
    VerifyDomainUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgOrganizationBrandingRepository, PgOrganizationDomainRepository, PgOrganizationPlanRepository,
    PgOrganizationRepository,
};

/// Hard-coded id of the "default" organization created by migration
/// `20260501000050_create_default_organization.sql`. Single-tenant deployments
/// (and v1.0 of any new install) use this org for every existing user/store
/// until v1.1's signup flow lets you spin up real orgs.
pub const DEFAULT_ORGANIZATION_ID_HEX: &str = "00000000-0000-0000-0000-000000000001";
