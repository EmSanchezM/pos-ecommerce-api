//! External delivery provider adapters.
//!
//! Mirrors the layout of `payments::infrastructure::gateways`: a trait, four
//! provider stubs (Hugo, PedidosYa, UberEats, Servientrega) returning a clear
//! `not yet wired` error, a fully working `Manual` adapter for offline
//! coordination, and a registry that dispatches by `DeliveryProviderType`.

mod hugo_adapter;
mod manual_external_adapter;
mod pedidos_ya_adapter;
mod provider_adapter;
mod registry;
mod servientrega_adapter;
mod uber_eats_adapter;

pub use hugo_adapter::HugoAdapter;
pub use manual_external_adapter::ManualExternalAdapter;
pub use pedidos_ya_adapter::PedidosYaAdapter;
pub use provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
pub use registry::{DefaultDeliveryProviderRegistry, DeliveryProviderRegistry};
pub use servientrega_adapter::ServientregaAdapter;
pub use uber_eats_adapter::UberEatsAdapter;
