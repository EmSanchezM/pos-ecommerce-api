use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Shipment;

#[derive(Debug, Deserialize)]
pub struct CreateShipmentCommand {
    pub sale_id: Uuid,
    pub store_id: Uuid,
    pub shipping_method_id: Uuid,
    pub shipping_cost: Decimal,
    pub currency: String,
    pub weight_kg: Option<Decimal>,
    pub recipient_name: String,
    pub recipient_phone: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: Option<String>,
    pub country: String,
    pub notes: Option<String>,
    /// When true, the matching pending payments::Transaction is auto-confirmed
    /// when this shipment reaches `delivered` (cash-on-delivery bridge).
    #[serde(default)]
    pub requires_cash_collection: bool,
    pub cash_amount: Option<Decimal>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AssignDriverCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub driver_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct DispatchProviderCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub delivery_provider_id: Option<Uuid>,
    /// Optional manual tracking number (Manual adapter scenario).
    pub manual_tracking_number: Option<String>,
    pub manual_carrier_name: Option<String>,
    pub estimated_delivery: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateShipmentStatusCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub status: String,
    pub notes: Option<String>,
    pub location_lat: Option<Decimal>,
    pub location_lng: Option<Decimal>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfirmPickupCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub pickup_code: String,
    pub picked_up_by_name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct MarkDeliveredCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub notes: Option<String>,
    pub location_lat: Option<Decimal>,
    pub location_lng: Option<Decimal>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MarkFailedCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct RescheduleShipmentCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub new_driver_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct CancelShipmentCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateTrackingCommand {
    #[serde(default)]
    pub shipment_id: Uuid,
    pub tracking_number: String,
    pub carrier_name: Option<String>,
    pub estimated_delivery: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListShipmentsQuery {
    pub store_id: Option<Uuid>,
    pub sale_id: Option<Uuid>,
    pub status: Option<String>,
    pub method_type: Option<String>,
    pub driver_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ShipmentResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub sale_id: Uuid,
    pub shipping_method_id: Uuid,
    pub method_type: String,
    pub driver_id: Option<Uuid>,
    pub delivery_provider_id: Option<Uuid>,
    pub status: String,
    pub tracking_number: Option<String>,
    pub carrier_name: Option<String>,
    pub shipping_cost: Decimal,
    pub currency: String,
    pub weight_kg: Option<Decimal>,
    pub pickup_code: Option<String>,
    pub pickup_ready_at: Option<DateTime<Utc>>,
    pub pickup_expires_at: Option<DateTime<Utc>>,
    pub picked_up_at: Option<DateTime<Utc>>,
    pub picked_up_by_name: Option<String>,
    pub requires_cash_collection: bool,
    pub cash_amount: Option<Decimal>,
    pub recipient_name: String,
    pub recipient_phone: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: Option<String>,
    pub country: String,
    pub notes: Option<String>,
    pub failure_reason: Option<String>,
    pub attempt_count: i32,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub estimated_delivery: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Shipment> for ShipmentResponse {
    fn from(s: Shipment) -> Self {
        Self {
            id: s.id().into_uuid(),
            store_id: s.store_id().into_uuid(),
            sale_id: s.sale_id().into_uuid(),
            shipping_method_id: s.shipping_method_id().into_uuid(),
            method_type: s.method_type().to_string(),
            driver_id: s.driver_id().map(|d| d.into_uuid()),
            delivery_provider_id: s.delivery_provider_id().map(|p| p.into_uuid()),
            status: s.status().to_string(),
            tracking_number: s.tracking_number().map(str::to_string),
            carrier_name: s.carrier_name().map(str::to_string),
            shipping_cost: s.shipping_cost(),
            currency: s.currency().to_string(),
            weight_kg: s.weight_kg(),
            pickup_code: s.pickup_code().map(str::to_string),
            pickup_ready_at: s.pickup_ready_at(),
            pickup_expires_at: s.pickup_expires_at(),
            picked_up_at: s.picked_up_at(),
            picked_up_by_name: s.picked_up_by_name().map(str::to_string),
            requires_cash_collection: s.requires_cash_collection(),
            cash_amount: s.cash_amount(),
            recipient_name: s.recipient_name().to_string(),
            recipient_phone: s.recipient_phone().map(str::to_string),
            address_line1: s.address_line1().to_string(),
            address_line2: s.address_line2().map(str::to_string),
            city: s.city().to_string(),
            state: s.state().to_string(),
            postal_code: s.postal_code().map(str::to_string),
            country: s.country().to_string(),
            notes: s.notes().map(str::to_string),
            failure_reason: s.failure_reason().map(str::to_string),
            attempt_count: s.attempt_count(),
            shipped_at: s.shipped_at(),
            delivered_at: s.delivered_at(),
            estimated_delivery: s.estimated_delivery(),
            cancelled_at: s.cancelled_at(),
            cancel_reason: s.cancel_reason().map(str::to_string),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShipmentListResponse {
    pub items: Vec<ShipmentResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Trimmed view used by the public `/track/{tracking_number}` endpoint.
/// Excludes recipient PII beyond what's already public (city/state).
#[derive(Debug, Serialize)]
pub struct PublicTrackingResponse {
    pub tracking_number: String,
    pub status: String,
    pub method_type: String,
    pub carrier_name: Option<String>,
    pub city: String,
    pub state: String,
    pub country: String,
    pub estimated_delivery: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub events: Vec<crate::application::dtos::TrackingEventResponse>,
}
