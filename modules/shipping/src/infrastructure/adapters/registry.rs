//! DeliveryProviderRegistry — selects an adapter for a given provider type.

use std::str::FromStr;
use std::sync::Arc;

use crate::ShippingError;
use crate::domain::value_objects::DeliveryProviderType;

use super::hugo_adapter::HugoAdapter;
use super::manual_external_adapter::ManualExternalAdapter;
use super::pedidos_ya_adapter::PedidosYaAdapter;
use super::provider_adapter::DeliveryProviderAdapter;
use super::servientrega_adapter::ServientregaAdapter;
use super::uber_eats_adapter::UberEatsAdapter;

pub trait DeliveryProviderRegistry: Send + Sync {
    fn for_type(&self, provider_type: DeliveryProviderType) -> Arc<dyn DeliveryProviderAdapter>;

    fn for_type_str(
        &self,
        provider_type: &str,
    ) -> Result<Arc<dyn DeliveryProviderAdapter>, ShippingError> {
        let pt = DeliveryProviderType::from_str(provider_type)?;
        Ok(self.for_type(pt))
    }
}

pub struct DefaultDeliveryProviderRegistry {
    manual: Arc<dyn DeliveryProviderAdapter>,
    hugo: Arc<dyn DeliveryProviderAdapter>,
    pedidos_ya: Arc<dyn DeliveryProviderAdapter>,
    uber_eats: Arc<dyn DeliveryProviderAdapter>,
    servientrega: Arc<dyn DeliveryProviderAdapter>,
}

impl DefaultDeliveryProviderRegistry {
    pub fn new() -> Self {
        Self {
            manual: Arc::new(ManualExternalAdapter::new()),
            hugo: Arc::new(HugoAdapter::new()),
            pedidos_ya: Arc::new(PedidosYaAdapter::new()),
            uber_eats: Arc::new(UberEatsAdapter::new()),
            servientrega: Arc::new(ServientregaAdapter::new()),
        }
    }
}

impl Default for DefaultDeliveryProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DeliveryProviderRegistry for DefaultDeliveryProviderRegistry {
    fn for_type(&self, provider_type: DeliveryProviderType) -> Arc<dyn DeliveryProviderAdapter> {
        match provider_type {
            DeliveryProviderType::Manual => self.manual.clone(),
            DeliveryProviderType::Hugo => self.hugo.clone(),
            DeliveryProviderType::PedidosYa => self.pedidos_ya.clone(),
            DeliveryProviderType::UberEats => self.uber_eats.clone(),
            DeliveryProviderType::Servientrega => self.servientrega.clone(),
        }
    }
}
