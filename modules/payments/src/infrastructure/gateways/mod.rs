//! Gateway adapters - infrastructure side of the payments module.
//!
//! Each external provider implements [`GatewayAdapter`]; the
//! [`GatewayAdapterRegistry`] dispatches a `GatewayType` to the right
//! adapter so use cases can stay provider-agnostic.
//!
//! The `Manual` adapter is the only one fully implemented today — it is the
//! workhorse in Honduras (transferencia / depósito en agencia / contra
//! entrega). Stripe / PayPal / BAC / Ficohsa are wired as stubs that report
//! a clear "not yet implemented" error so a misconfigured store fails loudly.

mod bac_credomatic_adapter;
mod ficohsa_adapter;
mod gateway_adapter;
mod manual_adapter;
mod paypal_adapter;
mod registry;
mod stripe_adapter;

pub use bac_credomatic_adapter::BacCredomaticAdapter;
pub use ficohsa_adapter::FicohsaAdapter;
pub use gateway_adapter::{GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent};
pub use manual_adapter::ManualGatewayAdapter;
pub use paypal_adapter::PayPalAdapter;
pub use registry::{DefaultGatewayAdapterRegistry, GatewayAdapterRegistry};
pub use stripe_adapter::StripeAdapter;
