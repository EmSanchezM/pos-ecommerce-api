use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::value_objects::{ServiceOrderId, ServiceOrderItemId, ServiceOrderItemType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrderItem {
    id: ServiceOrderItemId,
    service_order_id: ServiceOrderId,
    item_type: ServiceOrderItemType,
    description: String,
    quantity: Decimal,
    unit_price: Decimal,
    total: Decimal,
    product_id: Option<Uuid>,
    variant_id: Option<Uuid>,
    tax_rate: Decimal,
    tax_amount: Decimal,
    created_at: DateTime<Utc>,
}

impl ServiceOrderItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service_order_id: ServiceOrderId,
        item_type: ServiceOrderItemType,
        description: String,
        quantity: Decimal,
        unit_price: Decimal,
        product_id: Option<Uuid>,
        variant_id: Option<Uuid>,
        tax_rate: Decimal,
    ) -> Result<Self, ServiceOrdersError> {
        if description.trim().is_empty() {
            return Err(ServiceOrdersError::Validation(
                "item description is required".to_string(),
            ));
        }
        if quantity <= Decimal::ZERO {
            return Err(ServiceOrdersError::Validation(
                "quantity must be greater than 0".to_string(),
            ));
        }
        if unit_price < Decimal::ZERO {
            return Err(ServiceOrdersError::Validation(
                "unit_price cannot be negative".to_string(),
            ));
        }
        if tax_rate < Decimal::ZERO {
            return Err(ServiceOrdersError::Validation(
                "tax_rate cannot be negative".to_string(),
            ));
        }
        let subtotal = (quantity * unit_price).round_dp(4);
        let tax_amount = (subtotal * tax_rate).round_dp(4);
        let total = subtotal + tax_amount;
        Ok(Self {
            id: ServiceOrderItemId::new(),
            service_order_id,
            item_type,
            description,
            quantity,
            unit_price,
            total,
            product_id,
            variant_id,
            tax_rate,
            tax_amount,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ServiceOrderItemId,
        service_order_id: ServiceOrderId,
        item_type: ServiceOrderItemType,
        description: String,
        quantity: Decimal,
        unit_price: Decimal,
        total: Decimal,
        product_id: Option<Uuid>,
        variant_id: Option<Uuid>,
        tax_rate: Decimal,
        tax_amount: Decimal,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            service_order_id,
            item_type,
            description,
            quantity,
            unit_price,
            total,
            product_id,
            variant_id,
            tax_rate,
            tax_amount,
            created_at,
        }
    }

    pub fn id(&self) -> ServiceOrderItemId {
        self.id
    }
    pub fn service_order_id(&self) -> ServiceOrderId {
        self.service_order_id
    }
    pub fn item_type(&self) -> ServiceOrderItemType {
        self.item_type
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn quantity(&self) -> Decimal {
        self.quantity
    }
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }
    pub fn total(&self) -> Decimal {
        self.total
    }
    pub fn product_id(&self) -> Option<Uuid> {
        self.product_id
    }
    pub fn variant_id(&self) -> Option<Uuid> {
        self.variant_id
    }
    pub fn tax_rate(&self) -> Decimal {
        self.tax_rate
    }
    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
