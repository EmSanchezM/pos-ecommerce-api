use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::value_objects::{KdsItemStatus, KdsTicketId, KdsTicketItemId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdsTicketItem {
    id: KdsTicketItemId,
    ticket_id: KdsTicketId,
    sale_item_id: Option<Uuid>,
    product_id: Option<Uuid>,
    description: String,
    quantity: Decimal,
    modifiers_summary: String,
    special_instructions: Option<String>,
    status: KdsItemStatus,
    ready_at: Option<DateTime<Utc>>,
    served_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl KdsTicketItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ticket_id: KdsTicketId,
        sale_item_id: Option<Uuid>,
        product_id: Option<Uuid>,
        description: String,
        quantity: Decimal,
        modifiers_summary: String,
        special_instructions: Option<String>,
    ) -> Result<Self, RestaurantOperationsError> {
        if description.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "item description is required".to_string(),
            ));
        }
        if quantity <= Decimal::ZERO {
            return Err(RestaurantOperationsError::Validation(
                "quantity must be > 0".to_string(),
            ));
        }
        Ok(Self {
            id: KdsTicketItemId::new(),
            ticket_id,
            sale_item_id,
            product_id,
            description,
            quantity,
            modifiers_summary,
            special_instructions,
            status: KdsItemStatus::Pending,
            ready_at: None,
            served_at: None,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: KdsTicketItemId,
        ticket_id: KdsTicketId,
        sale_item_id: Option<Uuid>,
        product_id: Option<Uuid>,
        description: String,
        quantity: Decimal,
        modifiers_summary: String,
        special_instructions: Option<String>,
        status: KdsItemStatus,
        ready_at: Option<DateTime<Utc>>,
        served_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            ticket_id,
            sale_item_id,
            product_id,
            description,
            quantity,
            modifiers_summary,
            special_instructions,
            status,
            ready_at,
            served_at,
            created_at,
        }
    }

    pub fn transition_to(&mut self, to: KdsItemStatus) -> Result<(), RestaurantOperationsError> {
        if !self.status.can_transition_to(to) {
            return Err(RestaurantOperationsError::InvalidItemStateTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        let now = Utc::now();
        match to {
            KdsItemStatus::Ready => self.ready_at = Some(now),
            KdsItemStatus::Served => self.served_at = Some(now),
            _ => {}
        }
        Ok(())
    }

    pub fn id(&self) -> KdsTicketItemId {
        self.id
    }
    pub fn ticket_id(&self) -> KdsTicketId {
        self.ticket_id
    }
    pub fn sale_item_id(&self) -> Option<Uuid> {
        self.sale_item_id
    }
    pub fn product_id(&self) -> Option<Uuid> {
        self.product_id
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn quantity(&self) -> Decimal {
        self.quantity
    }
    pub fn modifiers_summary(&self) -> &str {
        &self.modifiers_summary
    }
    pub fn special_instructions(&self) -> Option<&str> {
        self.special_instructions.as_deref()
    }
    pub fn status(&self) -> KdsItemStatus {
        self.status
    }
    pub fn ready_at(&self) -> Option<DateTime<Utc>> {
        self.ready_at
    }
    pub fn served_at(&self) -> Option<DateTime<Utc>> {
        self.served_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
