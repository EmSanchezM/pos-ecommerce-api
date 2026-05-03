//! # Service Orders Module
//!
//! Vertical for **Talleres / Reparación** (mechanic, electronics, appliance
//! repair shops). Provides:
//!
//! - **Domain**: `Asset` (vehicle, equipment, appliance — owned by a
//!   customer), `ServiceOrder` (the workshop ticket aggregate root with the
//!   `Intake → Diagnosis → QuoteSent → QuoteApproved → InRepair → Testing →
//!   ReadyForPickup → Delivered | Canceled` workflow), `ServiceOrderItem`
//!   (labor + parts), `Diagnostic` (technician findings with severity), and
//!   `Quote` (versioned cost estimate with its own send/approve/reject
//!   workflow that drives the parent order's transitions).
//!
//! - **Application**: CRUD use cases for assets and items; intake/list/get
//!   for orders; quote workflow (create/send/approve/reject) that
//!   automatically marks older draft/sent quotes as superseded; service
//!   order transitions; passive subscriber; public status lookup by token.
//!
//! - **Infrastructure**: 5 `Pg*Repository` SQLx implementations.
//!
//! ## Decisions parked for v1.1+
//!
//! - **Inventory**: parts are recorded as items but stock is **not** decremented
//!   in v1.0. v1.1 wires `start_repair` (or `deliver`) into
//!   `inventory::AdjustStockUseCase`.
//! - **Sales**: `deliver` sets `generated_sale_id = None` in v1.0. v1.1
//!   invokes `sales::CreateSaleUseCase` and stores the resulting id.
//! - **WarrantyClaim**: deferred to v1.2.
//!
//! ## Public flow
//!
//! `GET /api/v1/public/service-orders/{id}?token=...` returns a curated
//! payload (status + items + latest diagnostic + latest quote) so the
//! customer can check progress without auth. Same `public_token` pattern as
//! `booking::Appointment`.
//!
//! See `docs/roadmap-modulos.md` (Fase 2.3 + "Plan detallado — Módulo
//! service_orders") for the full contract.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::ServiceOrdersError;

// Domain
pub use domain::entities::{Asset, Diagnostic, Quote, ServiceOrder, ServiceOrderItem};
pub use domain::repositories::{
    AssetRepository, DiagnosticRepository, ListServiceOrdersFilters, QuoteRepository,
    ServiceOrderItemRepository, ServiceOrderRepository,
};
pub use domain::value_objects::{
    AssetId, AssetType, DiagnosticId, DiagnosticSeverity, QuoteId, QuoteStatus, ServiceOrderId,
    ServiceOrderItemId, ServiceOrderItemType, ServiceOrderPriority, ServiceOrderStatus,
};

// Application
pub use application::dtos::{
    AddDiagnosticCommand, AddItemCommand, AssetResponse, CancelServiceOrderCommand,
    CreateQuoteCommand, DecideQuoteCommand, DiagnosticResponse, IntakeServiceOrderCommand,
    PublicServiceOrderResponse, QuoteResponse, RegisterAssetCommand, ServiceOrderDetailResponse,
    ServiceOrderItemResponse, ServiceOrderResponse, UpdateAssetCommand, UpdateItemCommand,
};
pub use application::subscriber::ServiceOrdersEventSubscriber;
pub use application::use_cases::{
    AddDiagnosticUseCase, AddItemUseCase, ApproveQuoteUseCase, CancelServiceOrderUseCase,
    CreateQuoteUseCase, DeactivateAssetUseCase, DeliverServiceOrderUseCase,
    DiagnoseServiceOrderUseCase, GetAssetUseCase, GetAssetWithHistoryUseCase,
    GetPublicServiceOrderUseCase, GetServiceOrderUseCase, IntakeServiceOrderUseCase,
    ListAssetsUseCase, ListDiagnosticsUseCase, ListItemsUseCase, ListQuotesUseCase,
    ListServiceOrdersUseCase, MarkReadyUseCase, RegisterAssetUseCase, RejectQuoteUseCase,
    RemoveItemUseCase, SendQuoteUseCase, StartRepairUseCase, StartTestingUseCase,
    UpdateAssetUseCase, UpdateItemUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgAssetRepository, PgDiagnosticRepository, PgQuoteRepository, PgServiceOrderItemRepository,
    PgServiceOrderRepository,
};
