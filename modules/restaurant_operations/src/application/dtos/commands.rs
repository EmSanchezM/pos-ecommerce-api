use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{Course, KdsItemStatus, TableStatus};

// -----------------------------------------------------------------------------
// Stations
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKitchenStationCommand {
    pub store_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKitchenStationCommand {
    pub name: String,
    pub color: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

// -----------------------------------------------------------------------------
// Tables
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRestaurantTableCommand {
    pub store_id: Uuid,
    pub label: String,
    pub capacity: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRestaurantTableCommand {
    pub label: String,
    pub capacity: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTableStatusCommand {
    pub status: TableStatus,
    /// Optional ticket id to attach when transitioning to `seated`/`reserved`.
    pub current_ticket_id: Option<Uuid>,
}

// -----------------------------------------------------------------------------
// Modifiers
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModifierGroupCommand {
    pub store_id: Uuid,
    pub name: String,
    pub min_select: i32,
    pub max_select: i32,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModifierGroupCommand {
    pub name: String,
    pub min_select: i32,
    pub max_select: i32,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModifierCommand {
    pub name: String,
    #[serde(default)]
    pub price_delta: Option<Decimal>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModifierCommand {
    pub name: String,
    #[serde(default)]
    pub price_delta: Option<Decimal>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignProductModifierGroupsCommand {
    pub group_ids: Vec<Uuid>,
}

// -----------------------------------------------------------------------------
// KDS tickets
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKdsTicketItemDto {
    pub description: String,
    pub quantity: Decimal,
    pub product_id: Option<Uuid>,
    pub sale_item_id: Option<Uuid>,
    /// Modifier ids selected for this item; the use case resolves their names
    /// into the item's `modifiers_summary` text.
    #[serde(default)]
    pub modifier_ids: Vec<Uuid>,
    pub special_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKdsTicketCommand {
    pub store_id: Uuid,
    pub station_id: Uuid,
    pub table_id: Option<Uuid>,
    pub sale_id: Option<Uuid>,
    #[serde(default)]
    pub course: Option<Course>,
    pub notes: Option<String>,
    pub items: Vec<CreateKdsTicketItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelKdsTicketCommand {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetItemStatusCommand {
    pub status: KdsItemStatus,
}
