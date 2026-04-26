//! # Shipping Module
//!
//! Shipping configuration and fulfillment for the POS + eCommerce platform,
//! covering three operational scenarios:
//!
//! - **StorePickup**: customer collects at the store with a pickup code
//! - **OwnDelivery**: store-owned drivers deliver
//! - **ExternalDelivery**: third-party couriers (Hugo, PedidosYa, Uber Eats,
//!   Servientrega) via [`DeliveryProviderAdapter`] implementations, with a
//!   `Manual` adapter for offline coordination
//!
//! Cross-cutting features:
//! - **Shipping zones + rate matrix** with weight/amount/time-of-day rules
//! - **Driver pool** with availability tracking
//! - **Tracking event log** (immutable audit trail)
//! - **Cash-on-delivery bridge**: when a shipment carrying a pending
//!   `payments::Transaction` is marked delivered, the transaction is
//!   automatically confirmed

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::ShippingError;

// -----------------------------------------------------------------------------
// Domain - Value Objects
// -----------------------------------------------------------------------------
pub use domain::value_objects::{
    DeliveryProviderId, DeliveryProviderType, DriverId, DriverStatus, ShipmentId, ShipmentStatus,
    ShipmentTrackingEventId, ShippingMethodId, ShippingMethodType, ShippingRateId,
    ShippingRateType, ShippingZoneId, TrackingEventSource, VehicleType,
};

// -----------------------------------------------------------------------------
// Domain - Entities
// -----------------------------------------------------------------------------
pub use domain::entities::{
    DeliveryProvider, Driver, Shipment, ShipmentTrackingEvent, ShippingMethod, ShippingRate,
    ShippingZone,
};

// -----------------------------------------------------------------------------
// Domain - Repository traits
// -----------------------------------------------------------------------------
pub use domain::repositories::{
    DeliveryProviderRepository, DriverRepository, ShipmentFilter, ShipmentRepository,
    ShipmentTrackingEventRepository, ShippingMethodRepository, ShippingRateRepository,
    ShippingZoneRepository,
};

// -----------------------------------------------------------------------------
// Application - DTOs + Use cases
// -----------------------------------------------------------------------------
pub use application::dtos::*;
pub use application::use_cases::*;

// -----------------------------------------------------------------------------
// Infrastructure - Adapters + Postgres repos
// -----------------------------------------------------------------------------
pub use infrastructure::adapters::{
    DefaultDeliveryProviderRegistry, DeliveryProviderAdapter, DeliveryProviderRegistry,
    DispatchRequest, DispatchResult, HugoAdapter, ManualExternalAdapter, PedidosYaAdapter,
    ProviderWebhookEvent, ServientregaAdapter, UberEatsAdapter,
};

pub use infrastructure::persistence::{
    PgDeliveryProviderRepository, PgDriverRepository, PgShipmentRepository,
    PgShipmentTrackingEventRepository, PgShippingMethodRepository, PgShippingRateRepository,
    PgShippingZoneRepository,
};
