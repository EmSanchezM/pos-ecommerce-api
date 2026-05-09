//! # Subscriptions Module
//!
//! SaaS billing of the platform itself: charges every `Organization` a
//! recurring fee for using the system. The `SubscriptionPlan` carries the
//! **price + cadence**; `tenancy::OrganizationPlan` keeps the **feature
//! flags + tier limits**. They are linked through the `tier` field — when
//! a `Subscription` activates or changes plans, a tenancy subscriber syncs
//! `OrganizationPlan.tier` and snaps the feature flags to the tier defaults.
//!
//! See `docs/roadmap-modulos.md` (Fase 3.2 + "Plan detallado — Módulo
//! subscriptions") for the full v1.0 contract.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::SubscriptionError;

// Domain re-exports
pub use domain::entities::{BillingCycle, DunningAttempt, Subscription, SubscriptionPlan};
pub use domain::repositories::{
    BillingCycleRepository, DunningAttemptRepository, SubscriptionPlanRepository,
    SubscriptionRepository,
};
pub use domain::value_objects::{
    BillingCycleId, BillingCycleStatus, BillingInterval, DunningAttemptId, DunningOutcome,
    PlanCode, SubscriptionId, SubscriptionPlanId, SubscriptionStatus,
};

// Application re-exports
pub use application::dtos::{
    BillingCycleResponse, BillingTickReport, CancelSubscriptionCommand, ChangePlanCommand,
    CreatePlanCommand, DunningAttemptResponse, ListBillingCyclesQuery, ListPlansQuery,
    PaginatedBillingCycles, PaginatedPlans, PlanResponse, ResumeSubscriptionCommand,
    SubscribeOrganizationCommand, SubscriptionResponse, UpdatePlanCommand,
};
pub use application::subscriber::SubscriptionsEventSubscriber;
pub use application::use_cases::{
    CancelSubscriptionUseCase, ChangePlanUseCase, CreatePlanUseCase, DeactivatePlanUseCase,
    GRACE_PERIOD_DAYS, GetPlanUseCase, GetSubscriptionUseCase, ListBillingCyclesUseCase,
    ListPlansUseCase, PaymentOutcome, ProcessBillingCycleUseCase, ProcessDunningAttemptUseCase,
    RecordPaymentOutcomeUseCase, ResumeSubscriptionUseCase, RunBillingTickUseCase,
    SubscribeOrganizationUseCase, TickPastDueSubscriptionsUseCase, UpdatePlanUseCase,
};

// Infrastructure re-exports
pub use infrastructure::persistence::{
    PgBillingCycleRepository, PgDunningAttemptRepository, PgSubscriptionPlanRepository,
    PgSubscriptionRepository,
};
pub use infrastructure::{
    BillingInvoiceGateway, BillingPaymentGateway, ChargeCreated, InvoiceCreated,
};
