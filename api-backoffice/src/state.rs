// BackofficeAppState — DI wiring for the backoffice binary
//
// Holds all shared dependencies for the backoffice API.
// Mirrors api-gateway/src/state.rs but scoped to backoffice concerns only.

use std::sync::Arc;

// BackofficeAuditSubscriber and PgBackofficeAuditLogRepository will be used
// in P4-T08 when the event_dispatcher job is wired.
use analytics::{KpiSnapshotRepository, PgKpiSnapshotRepository};
#[allow(unused_imports)]
use audit_infra::{BackofficeAuditSubscriber, PgBackofficeAuditLogRepository};
use backoffice_identity::{
    AuthenticateBackofficeUserUseCase, BackofficeTokenService, BackofficeUserRepository,
    IssueImpersonationTokenWithAuditUseCase, JwtBackofficeTokenService, PgBackofficeUserRepository,
    SuspendOrganizationWithAuditUseCase,
};
use events::{PgOutboxRepository, PublishEventUseCase};
use identity::{PgUserRepository, UserRepository};
use sqlx::PgPool;
use subscriptions::{
    BillingCycleRepository, BillingPaymentGateway, DunningAttemptRepository,
    PgBillingCycleRepository, PgDunningAttemptRepository, PgSubscriptionPlanRepository,
    PgSubscriptionRepository, SubscriptionPlanRepository, SubscriptionRepository,
};

use subscription_billing::StubBillingPaymentGateway;
use tenancy::{OrganizationRepository, PgOrganizationRepository};

/// Application state shared across all backoffice HTTP handlers.
///
/// Follows the same pattern as `api-gateway/src/state.rs`.
#[derive(Clone)]
pub struct BackofficeAppState {
    /// Direct access to the PostgreSQL connection pool for transactional operations.
    pool: PgPool,
    /// Backoffice user repository.
    user_repo: Arc<dyn BackofficeUserRepository>,
    /// Tenant user repository — for verifying that a target tenant_user_id exists
    /// before issuing an impersonation token (P5-T03).
    tenant_user_repo: Arc<dyn UserRepository>,
    /// Token service for validating incoming backoffice JWTs in the auth middleware.
    token_service: Arc<dyn BackofficeTokenService>,
    /// Use case: authenticate a backoffice user and issue a JWT.
    authenticate_use_case: Arc<AuthenticateBackofficeUserUseCase>,
    /// Organization repository (tenancy module) — for suspend/activate operations.
    org_repo: Arc<dyn OrganizationRepository>,
    /// Suspend org use case with transactional audit (backoffice_identity module).
    suspend_with_audit_use_case: Arc<SuspendOrganizationWithAuditUseCase>,
    /// Impersonation use case — issues tenant-scoped token + writes audit event.
    impersonation_use_case: Arc<IssueImpersonationTokenWithAuditUseCase>,
    /// Publish event use case — writes to outbox in a transaction.
    publish_event: Arc<PublishEventUseCase>,
    /// Subscription plan repository (subscriptions module) — backs plan CRUD.
    subscription_plan_repo: Arc<dyn SubscriptionPlanRepository>,
    /// Subscription repository — backs subscription admin (force-cancel,
    /// change-plan, resume).
    subscription_repo: Arc<dyn SubscriptionRepository>,
    /// Billing-cycle repository — needed by manual dunning trigger.
    billing_cycle_repo: Arc<dyn BillingCycleRepository>,
    /// Dunning-attempt repository — backs manual dunning trigger.
    dunning_repo: Arc<dyn DunningAttemptRepository>,
    /// Payment gateway used when manually firing a dunning attempt. v1.0 stub.
    dunning_payment_gateway: Arc<dyn BillingPaymentGateway>,
    /// KPI snapshot repository — backs cross-org (system-wide) analytics reads.
    kpi_snapshot_repo: Arc<dyn KpiSnapshotRepository>,
}

impl BackofficeAppState {
    /// Construct BackofficeAppState from a pool and JWT secrets.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `backoffice_secret` - Secret for signing/validating backoffice JWTs
    /// * `backoffice_issuer` - Issuer string embedded in backoffice tokens
    /// * `tenant_secret` - JWT_SECRET for signing impersonation tokens (Decision 2)
    pub fn from_pool(
        pool: PgPool,
        backoffice_secret: String,
        backoffice_issuer: String,
        tenant_secret: String,
    ) -> Self {
        let pool_arc = Arc::new(pool.clone());

        let user_repo: Arc<dyn BackofficeUserRepository> =
            Arc::new(PgBackofficeUserRepository::new((*pool_arc).clone()));

        let tenant_user_repo: Arc<dyn UserRepository> =
            Arc::new(PgUserRepository::new((*pool_arc).clone()));

        let token_service = Arc::new(JwtBackofficeTokenService::with_issuer(
            backoffice_secret,
            backoffice_issuer,
        ));

        let authenticate_use_case = Arc::new(AuthenticateBackofficeUserUseCase::new(
            user_repo.clone(),
            token_service.clone(),
        ));

        let org_repo: Arc<dyn OrganizationRepository> =
            Arc::new(PgOrganizationRepository::new((*pool_arc).clone()));

        let outbox_repo = Arc::new(PgOutboxRepository::new((*pool_arc).clone()));
        let publish_event = Arc::new(PublishEventUseCase::new(outbox_repo));

        let suspend_with_audit_use_case = Arc::new(SuspendOrganizationWithAuditUseCase::new(
            (*pool_arc).clone(),
            org_repo.clone(),
            publish_event.clone(),
        ));

        let impersonation_use_case = Arc::new(IssueImpersonationTokenWithAuditUseCase::new(
            (*pool_arc).clone(),
            user_repo.clone(),
            token_service.clone(),
            publish_event.clone(),
            tenant_secret,
        ));

        let subscription_plan_repo: Arc<dyn SubscriptionPlanRepository> =
            Arc::new(PgSubscriptionPlanRepository::new((*pool_arc).clone()));

        let subscription_repo: Arc<dyn SubscriptionRepository> =
            Arc::new(PgSubscriptionRepository::new((*pool_arc).clone()));

        let billing_cycle_repo: Arc<dyn BillingCycleRepository> =
            Arc::new(PgBillingCycleRepository::new((*pool_arc).clone()));

        let dunning_repo: Arc<dyn DunningAttemptRepository> =
            Arc::new(PgDunningAttemptRepository::new((*pool_arc).clone()));

        let dunning_payment_gateway: Arc<dyn BillingPaymentGateway> =
            Arc::new(StubBillingPaymentGateway::new());

        let kpi_snapshot_repo: Arc<dyn KpiSnapshotRepository> =
            Arc::new(PgKpiSnapshotRepository::new((*pool_arc).clone()));

        Self {
            pool,
            user_repo,
            tenant_user_repo,
            token_service,
            authenticate_use_case,
            org_repo,
            suspend_with_audit_use_case,
            impersonation_use_case,
            publish_event,
            subscription_plan_repo,
            subscription_repo,
            billing_cycle_repo,
            dunning_repo,
            dunning_payment_gateway,
            kpi_snapshot_repo,
        }
    }

    /// Returns a reference to the PostgreSQL connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Returns the backoffice user repository.
    pub fn user_repo(&self) -> Arc<dyn BackofficeUserRepository> {
        self.user_repo.clone()
    }

    /// Returns the tenant user repository (for tenant_user_id existence checks).
    pub fn tenant_user_repo(&self) -> Arc<dyn UserRepository> {
        self.tenant_user_repo.clone()
    }

    /// Returns the backoffice token service (used by auth middleware for validation).
    pub fn token_service(&self) -> Arc<dyn BackofficeTokenService> {
        self.token_service.clone()
    }

    /// Returns the authenticate backoffice user use case.
    pub fn authenticate_use_case(&self) -> Arc<AuthenticateBackofficeUserUseCase> {
        self.authenticate_use_case.clone()
    }

    /// Returns the organization repository (for suspend/activate).
    pub fn org_repo(&self) -> Arc<dyn OrganizationRepository> {
        self.org_repo.clone()
    }

    /// Returns the suspend organization with audit use case.
    pub fn suspend_with_audit_use_case(&self) -> Arc<SuspendOrganizationWithAuditUseCase> {
        self.suspend_with_audit_use_case.clone()
    }

    /// Returns the impersonation use case.
    pub fn impersonation_use_case(&self) -> Arc<IssueImpersonationTokenWithAuditUseCase> {
        self.impersonation_use_case.clone()
    }

    /// Returns the publish event use case (writes outbox events in a transaction).
    pub fn publish_event(&self) -> Arc<PublishEventUseCase> {
        self.publish_event.clone()
    }

    /// Returns the subscription plan repository (backs plan CRUD).
    pub fn subscription_plan_repo(&self) -> Arc<dyn SubscriptionPlanRepository> {
        self.subscription_plan_repo.clone()
    }

    /// Returns the subscription repository (backs subscription admin).
    pub fn subscription_repo(&self) -> Arc<dyn SubscriptionRepository> {
        self.subscription_repo.clone()
    }

    /// Returns the billing-cycle repository.
    pub fn billing_cycle_repo(&self) -> Arc<dyn BillingCycleRepository> {
        self.billing_cycle_repo.clone()
    }

    /// Returns the dunning-attempt repository.
    pub fn dunning_repo(&self) -> Arc<dyn DunningAttemptRepository> {
        self.dunning_repo.clone()
    }

    /// Returns the payment gateway used for manual dunning triggers.
    pub fn dunning_payment_gateway(&self) -> Arc<dyn BillingPaymentGateway> {
        self.dunning_payment_gateway.clone()
    }

    /// Returns the KPI snapshot repository (backs cross-org analytics reads).
    pub fn kpi_snapshot_repo(&self) -> Arc<dyn KpiSnapshotRepository> {
        self.kpi_snapshot_repo.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P3-T01: BackofficeAppState::from_pool constructs without panic and
    /// exposes the expected use cases and pool.
    ///
    /// We create a PgPool pointing to a fictional URL — this test verifies
    /// that the state object is correctly wired at the type level without
    /// actually connecting to a database. We use `PgPool::connect_lazy` so
    /// no real connection is attempted.
    #[tokio::test]
    async fn backoffice_app_state_constructs_and_exposes_use_case() {
        // lazy connect — no real DB needed
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let state = BackofficeAppState::from_pool(
            pool,
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
            "tenant-secret-at-least-32-bytes-long-x".to_string(),
        );

        // authenticate_use_case is Arc-wrapped and clone-able
        let _uc = state.authenticate_use_case();
        // pool is accessible
        let _pool = state.pool();
        // user_repo is accessible
        let _repo = state.user_repo();
        // impersonation_use_case is accessible (P5)
        let _imp = state.impersonation_use_case();
        // subscription_plan_repo is wired (P6 — plan CRUD)
        let _plans = state.subscription_plan_repo();
        // subscription_repo is wired (P6 — subscription admin)
        let _subs = state.subscription_repo();
        // dunning wiring (P6 — manual dunning trigger)
        let _cyc = state.billing_cycle_repo();
        let _dun = state.dunning_repo();
        let _gw = state.dunning_payment_gateway();
        // analytics wiring (P6 — cross-org KPI reads)
        let _kpi = state.kpi_snapshot_repo();
    }

    #[tokio::test]
    async fn backoffice_app_state_is_clone() {
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let state = BackofficeAppState::from_pool(
            pool,
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
            "tenant-secret-at-least-32-bytes-long-x".to_string(),
        );

        // State must be Clone so Axum can share it across handlers.
        let _clone = state.clone();
    }
}
