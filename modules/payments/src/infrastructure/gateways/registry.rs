//! GatewayAdapterRegistry — selects the right adapter for a given gateway.
//!
//! Use cases call `registry.for_type(gateway.gateway_type())` to obtain the
//! adapter that should handle a charge/refund/webhook for that gateway. This
//! is the seam where future provider implementations get plugged in.

use std::str::FromStr;
use std::sync::Arc;

use crate::PaymentsError;
use crate::domain::value_objects::GatewayType;

use super::bac_credomatic_adapter::BacCredomaticAdapter;
use super::ficohsa_adapter::FicohsaAdapter;
use super::gateway_adapter::GatewayAdapter;
use super::manual_adapter::ManualGatewayAdapter;
use super::paypal_adapter::PayPalAdapter;
use super::stripe_adapter::StripeAdapter;

pub trait GatewayAdapterRegistry: Send + Sync {
    fn for_type(&self, gateway_type: GatewayType) -> Arc<dyn GatewayAdapter>;

    /// Convenience for handlers that receive the type as a path segment.
    fn for_type_str(&self, gateway_type: &str) -> Result<Arc<dyn GatewayAdapter>, PaymentsError> {
        let gt = GatewayType::from_str(gateway_type)?;
        Ok(self.for_type(gt))
    }
}

/// Default registry: holds one shared instance per provider. Stub adapters
/// are pre-wired for Stripe / PayPal / BAC / Ficohsa; only `Manual` actually
/// processes today.
pub struct DefaultGatewayAdapterRegistry {
    manual: Arc<dyn GatewayAdapter>,
    stripe: Arc<dyn GatewayAdapter>,
    paypal: Arc<dyn GatewayAdapter>,
    bac_credomatic: Arc<dyn GatewayAdapter>,
    ficohsa: Arc<dyn GatewayAdapter>,
}

impl DefaultGatewayAdapterRegistry {
    pub fn new() -> Self {
        Self {
            manual: Arc::new(ManualGatewayAdapter::new()),
            stripe: Arc::new(StripeAdapter::new()),
            paypal: Arc::new(PayPalAdapter::new()),
            bac_credomatic: Arc::new(BacCredomaticAdapter::new()),
            ficohsa: Arc::new(FicohsaAdapter::new()),
        }
    }

    /// Build a registry where every provider points to a custom adapter.
    /// Use this in tests to inject mocks per provider.
    pub fn with_overrides(
        manual: Arc<dyn GatewayAdapter>,
        stripe: Arc<dyn GatewayAdapter>,
        paypal: Arc<dyn GatewayAdapter>,
        bac_credomatic: Arc<dyn GatewayAdapter>,
        ficohsa: Arc<dyn GatewayAdapter>,
    ) -> Self {
        Self {
            manual,
            stripe,
            paypal,
            bac_credomatic,
            ficohsa,
        }
    }
}

impl Default for DefaultGatewayAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayAdapterRegistry for DefaultGatewayAdapterRegistry {
    fn for_type(&self, gateway_type: GatewayType) -> Arc<dyn GatewayAdapter> {
        match gateway_type {
            GatewayType::Manual => self.manual.clone(),
            GatewayType::Stripe => self.stripe.clone(),
            GatewayType::PayPal => self.paypal.clone(),
            GatewayType::BacCredomatic => self.bac_credomatic.clone(),
            GatewayType::Ficohsa => self.ficohsa.clone(),
        }
    }
}
