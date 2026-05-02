//! # Demand Planning Module
//!
//! Forecasting + automatic replenishment for the POS + eCommerce platform.
//!
//! - **Domain**: `DemandForecast`, `ReorderPolicy`, `ReplenishmentSuggestion`,
//!   `AbcClassification`. Optimistic locking on `ReorderPolicy::version`.
//! - **Application**: pure forecasting math (`forecasting/` — moving average,
//!   exponential smoothing, Holt-Winters, outlier filter), use cases for
//!   recompute/generate/approve/dismiss/classify, and a
//!   `DemandPlanningEventSubscriber` that observes inventory and sales events.
//! - **Infrastructure**: `Pg*Repository` implementations and read-only
//!   projections over `sales` / `inventory_stock`.
//!
//! Forecasting runs entirely in Rust with `statrs`; there are no external AI
//! adapters in v1 or v2 by design (cost decision — see `docs/roadmap-modulos.md`).
//!
//! See `docs/roadmap-modulos.md` ("Plan detallado — Módulo `demand_planning`").

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::DemandPlanningError;

// Domain
pub use domain::entities::{
    AbcClassification, DemandForecast, ReorderPolicy, ReplenishmentSuggestion, SeriesPoint,
};
pub use domain::repositories::{
    AbcClassificationRepository, DemandForecastRepository, ReorderPolicyRepository,
    ReplenishmentSuggestionRepository, RevenueRow, SalesHistoryRepository, StockSnapshot,
    StockSnapshotRepository,
};
pub use domain::value_objects::{
    AbcClass, AbcClassificationId, ForecastId, ForecastMethod, ForecastPeriod, ReorderPolicyId,
    SuggestionId, SuggestionStatus,
};

// Application
pub use application::dtos::{
    AbcClassificationResponse, ApproveSuggestionCommand, DemandForecastResponse,
    DismissSuggestionCommand, ReorderPolicyResponse, ReplenishmentSuggestionResponse,
    UpsertReorderPolicyCommand,
};
pub use application::subscriber::DemandPlanningEventSubscriber;
pub use application::use_cases::{
    ApproveSuggestionUseCase, ClassifyAbcUseCase, DismissSuggestionUseCase,
    GenerateReplenishmentSuggestionsUseCase, GetForecastUseCase, ListAbcClassificationsUseCase,
    ListReorderPoliciesUseCase, ListReplenishmentSuggestionsUseCase, RecomputeForecastUseCase,
    UpsertReorderPolicyUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgAbcClassificationRepository, PgDemandForecastRepository, PgReorderPolicyRepository,
    PgReplenishmentSuggestionRepository, PgSalesHistoryRepository, PgStockSnapshotRepository,
};
