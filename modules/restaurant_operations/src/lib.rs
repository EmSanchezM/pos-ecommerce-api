//! # Restaurant Operations Module
//!
//! Vertical for **Restaurantes / F&B**. Provides:
//!
//! - **Domain**: `KitchenStation` (Hot Line / Cold Line / Bar),
//!   `RestaurantTable`, `MenuModifierGroup` + `MenuModifier` (with
//!   product M2M), `KdsTicket` (kitchen display ticket aggregate root with
//!   `Pending → InProgress → Ready → Served | Canceled` workflow), and
//!   `KdsTicketItem` mirroring the per-item lifecycle.
//!
//! - **Application**: CRUD use cases for stations / tables / modifiers,
//!   `CreateKdsTicketUseCase` (resolves modifier ids into a summary text and
//!   auto-assigns a per-store ticket number), the ticket lifecycle
//!   (`Send`/`Ready`/`Serve`/`Cancel`), and `SetItemStatusUseCase` which
//!   auto-advances the parent ticket when every item reaches the same
//!   status. Every state-changing use case publishes a `KdsEvent` to the
//!   `KdsBroadcaster` so the SSE handler in the API gateway can fan it out
//!   to every connected kitchen display.
//!
//! - **Infrastructure**: 5 `Pg*Repository` SQLx implementations plus
//!   `TokioBroadcastKdsBroadcaster` (one `tokio::sync::broadcast` channel
//!   per station, lazily created).
//!
//! ## Decisions parked for v1.1+
//!
//! - **`FloorPlan`** with 2D coordinates — the table model is ready, the
//!   floor-plan layout is a v1.2 add-on.
//! - **`sale.item_added` subscriber** — v1.1 will auto-create / extend KDS
//!   tickets routed to the right station based on a configurable mapping.
//! - **`Tip` / `TipDistribution`, `SplitBill`, `TableReservation`** — v1.1+.
//! - **Course-based ticket splitting** — v1.2.
//!
//! See `docs/roadmap-modulos.md` (Fase 2.1 + "Plan detallado — Módulo
//! restaurant_operations") for the full contract.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::RestaurantOperationsError;

// Domain
pub use domain::entities::{
    KdsTicket, KdsTicketItem, KitchenStation, MenuModifier, MenuModifierGroup, RestaurantTable,
};
pub use domain::repositories::{
    KdsTicketItemRepository, KdsTicketRepository, KitchenStationRepository, ListKdsTicketsFilters,
    MenuModifierRepository, ModifierGroupWithModifiers, RestaurantTableRepository,
};
pub use domain::value_objects::{
    Course, KdsItemStatus, KdsTicketId, KdsTicketItemId, KdsTicketStatus, KitchenStationId,
    MenuModifierGroupId, MenuModifierId, RestaurantTableId, TableStatus,
};

// Application
pub use application::broadcaster::{KdsBroadcaster, KdsEvent, NoopKdsBroadcaster};
pub use application::dtos::{
    AssignProductModifierGroupsCommand, CancelKdsTicketCommand, CreateKdsTicketCommand,
    CreateKdsTicketItemDto, CreateKitchenStationCommand, CreateModifierCommand,
    CreateModifierGroupCommand, CreateRestaurantTableCommand, KdsTicketDetailResponse,
    KdsTicketItemResponse, KdsTicketResponse, KitchenStationResponse, MenuModifierGroupResponse,
    MenuModifierResponse, RestaurantTableResponse, SetItemStatusCommand, SetTableStatusCommand,
    UpdateKitchenStationCommand, UpdateModifierCommand, UpdateModifierGroupCommand,
    UpdateRestaurantTableCommand,
};
pub use application::subscriber::RestaurantOperationsEventSubscriber;
pub use application::use_cases::{
    AddModifierUseCase, AssignProductModifierGroupsUseCase, CancelKdsTicketUseCase,
    CreateKdsTicketUseCase, CreateKitchenStationUseCase, CreateModifierGroupUseCase,
    CreateRestaurantTableUseCase, DeactivateKitchenStationUseCase,
    DeactivateRestaurantTableUseCase, GetKdsTicketUseCase, GetProductModifierGroupsUseCase,
    KdsDeps, ListGroupsWithModifiersUseCase, ListKdsTicketsUseCase, ListKitchenStationsUseCase,
    ListRestaurantTablesUseCase, MarkKdsTicketReadyUseCase, SendKdsTicketUseCase,
    ServeKdsTicketUseCase, SetItemStatusUseCase, SetTableStatusUseCase,
    UpdateKitchenStationUseCase, UpdateModifierGroupUseCase, UpdateModifierUseCase,
    UpdateRestaurantTableUseCase,
};

// Infrastructure
pub use infrastructure::broadcaster::TokioBroadcastKdsBroadcaster;
pub use infrastructure::persistence::{
    PgKdsTicketItemRepository, PgKdsTicketRepository, PgKitchenStationRepository,
    PgMenuModifierRepository, PgRestaurantTableRepository,
};
