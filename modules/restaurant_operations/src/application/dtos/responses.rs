use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{
    KdsTicket, KdsTicketItem, KitchenStation, MenuModifier, MenuModifierGroup, RestaurantTable,
};
use crate::domain::repositories::ModifierGroupWithModifiers;
use crate::domain::value_objects::{Course, KdsItemStatus, KdsTicketStatus, TableStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenStationResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&KitchenStation> for KitchenStationResponse {
    fn from(s: &KitchenStation) -> Self {
        Self {
            id: s.id().into_uuid(),
            store_id: s.store_id(),
            name: s.name().to_string(),
            color: s.color().map(String::from),
            sort_order: s.sort_order(),
            is_active: s.is_active(),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestaurantTableResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub label: String,
    pub capacity: i32,
    pub status: TableStatus,
    pub current_ticket_id: Option<Uuid>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&RestaurantTable> for RestaurantTableResponse {
    fn from(t: &RestaurantTable) -> Self {
        Self {
            id: t.id().into_uuid(),
            store_id: t.store_id(),
            label: t.label().to_string(),
            capacity: t.capacity(),
            status: t.status(),
            current_ticket_id: t.current_ticket_id(),
            notes: t.notes().map(String::from),
            is_active: t.is_active(),
            created_at: t.created_at(),
            updated_at: t.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuModifierResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub price_delta: Decimal,
    pub sort_order: i32,
    pub is_active: bool,
}

impl From<&MenuModifier> for MenuModifierResponse {
    fn from(m: &MenuModifier) -> Self {
        Self {
            id: m.id().into_uuid(),
            group_id: m.group_id().into_uuid(),
            name: m.name().to_string(),
            price_delta: m.price_delta(),
            sort_order: m.sort_order(),
            is_active: m.is_active(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuModifierGroupResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub min_select: i32,
    pub max_select: i32,
    pub sort_order: i32,
    pub is_active: bool,
    pub modifiers: Vec<MenuModifierResponse>,
}

impl From<&MenuModifierGroup> for MenuModifierGroupResponse {
    fn from(g: &MenuModifierGroup) -> Self {
        Self {
            id: g.id().into_uuid(),
            store_id: g.store_id(),
            name: g.name().to_string(),
            min_select: g.min_select(),
            max_select: g.max_select(),
            sort_order: g.sort_order(),
            is_active: g.is_active(),
            modifiers: Vec::new(),
        }
    }
}

impl From<&ModifierGroupWithModifiers> for MenuModifierGroupResponse {
    fn from(m: &ModifierGroupWithModifiers) -> Self {
        let mut response = MenuModifierGroupResponse::from(&m.group);
        response.modifiers = m.modifiers.iter().map(MenuModifierResponse::from).collect();
        response
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdsTicketItemResponse {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub sale_item_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: Decimal,
    pub modifiers_summary: String,
    pub special_instructions: Option<String>,
    pub status: KdsItemStatus,
    pub ready_at: Option<DateTime<Utc>>,
    pub served_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<&KdsTicketItem> for KdsTicketItemResponse {
    fn from(i: &KdsTicketItem) -> Self {
        Self {
            id: i.id().into_uuid(),
            ticket_id: i.ticket_id().into_uuid(),
            sale_item_id: i.sale_item_id(),
            product_id: i.product_id(),
            description: i.description().to_string(),
            quantity: i.quantity(),
            modifiers_summary: i.modifiers_summary().to_string(),
            special_instructions: i.special_instructions().map(String::from),
            status: i.status(),
            ready_at: i.ready_at(),
            served_at: i.served_at(),
            created_at: i.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdsTicketResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub station_id: Uuid,
    pub table_id: Option<Uuid>,
    pub sale_id: Option<Uuid>,
    pub ticket_number: i32,
    pub status: KdsTicketStatus,
    pub course: Course,
    pub notes: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub ready_at: Option<DateTime<Utc>>,
    pub served_at: Option<DateTime<Utc>>,
    pub canceled_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&KdsTicket> for KdsTicketResponse {
    fn from(t: &KdsTicket) -> Self {
        Self {
            id: t.id().into_uuid(),
            store_id: t.store_id(),
            station_id: t.station_id().into_uuid(),
            table_id: t.table_id().map(|i| i.into_uuid()),
            sale_id: t.sale_id(),
            ticket_number: t.ticket_number(),
            status: t.status(),
            course: t.course(),
            notes: t.notes().map(String::from),
            sent_at: t.sent_at(),
            ready_at: t.ready_at(),
            served_at: t.served_at(),
            canceled_reason: t.canceled_reason().map(String::from),
            created_by: t.created_by(),
            created_at: t.created_at(),
            updated_at: t.updated_at(),
        }
    }
}

/// Detail returned by `GET /kds/tickets/{id}` and the create endpoint —
/// bundles the ticket with its items so the kitchen display renders the full
/// card without an extra call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdsTicketDetailResponse {
    #[serde(flatten)]
    pub ticket: KdsTicketResponse,
    pub items: Vec<KdsTicketItemResponse>,
}
