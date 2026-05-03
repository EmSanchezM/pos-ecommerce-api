//! Service order workflow transitions (excluding the quote-driven steps,
//! which live in `quote_use_cases.rs`). v1.0 uses cases just toggle state.
//! v1.1 will hook `start_repair` into `inventory::AdjustStockUseCase` and
//! `deliver` into `sales::CreateSaleUseCase`.

use std::sync::Arc;

use crate::ServiceOrdersError;
use crate::application::dtos::CancelServiceOrderCommand;
use crate::domain::entities::ServiceOrder;
use crate::domain::repositories::ServiceOrderRepository;
use crate::domain::value_objects::ServiceOrderId;

macro_rules! single_transition_use_case {
    ($name:ident, $method:ident) => {
        pub struct $name {
            orders: Arc<dyn ServiceOrderRepository>,
        }

        impl $name {
            pub fn new(orders: Arc<dyn ServiceOrderRepository>) -> Self {
                Self { orders }
            }

            pub async fn execute(
                &self,
                id: ServiceOrderId,
            ) -> Result<ServiceOrder, ServiceOrdersError> {
                let mut order = self
                    .orders
                    .find_by_id(id)
                    .await?
                    .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(id.into_uuid()))?;
                order.$method()?;
                self.orders.update(&order).await?;
                Ok(order)
            }
        }
    };
}

single_transition_use_case!(DiagnoseServiceOrderUseCase, diagnose);
single_transition_use_case!(StartRepairUseCase, start_repair);
single_transition_use_case!(StartTestingUseCase, start_testing);
single_transition_use_case!(MarkReadyUseCase, mark_ready);

pub struct DeliverServiceOrderUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
}

impl DeliverServiceOrderUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { orders }
    }

    pub async fn execute(&self, id: ServiceOrderId) -> Result<ServiceOrder, ServiceOrdersError> {
        let mut order = self
            .orders
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(id.into_uuid()))?;
        // v1.0: pass `None` for generated_sale_id. v1.1 will invoke
        // `sales::CreateSaleUseCase` here and pass the resulting id.
        order.deliver(None)?;
        self.orders.update(&order).await?;
        Ok(order)
    }
}

pub struct CancelServiceOrderUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
}

impl CancelServiceOrderUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { orders }
    }

    pub async fn execute(
        &self,
        id: ServiceOrderId,
        cmd: CancelServiceOrderCommand,
    ) -> Result<ServiceOrder, ServiceOrdersError> {
        let mut order = self
            .orders
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(id.into_uuid()))?;
        order.cancel(cmd.reason)?;
        self.orders.update(&order).await?;
        Ok(order)
    }
}
