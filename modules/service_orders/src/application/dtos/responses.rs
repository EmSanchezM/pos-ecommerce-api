use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::entities::{Asset, Diagnostic, Quote, ServiceOrder, ServiceOrderItem};
use crate::domain::value_objects::{
    AssetType, DiagnosticSeverity, QuoteStatus, ServiceOrderItemType, ServiceOrderPriority,
    ServiceOrderStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub asset_type: AssetType,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub identifier: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub attributes: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Asset> for AssetResponse {
    fn from(a: &Asset) -> Self {
        Self {
            id: a.id().into_uuid(),
            store_id: a.store_id(),
            customer_id: a.customer_id(),
            asset_type: a.asset_type(),
            brand: a.brand().map(String::from),
            model: a.model().map(String::from),
            identifier: a.identifier().map(String::from),
            year: a.year(),
            color: a.color().map(String::from),
            description: a.description().map(String::from),
            attributes: a.attributes().clone(),
            is_active: a.is_active(),
            created_at: a.created_at(),
            updated_at: a.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrderResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub asset_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub status: ServiceOrderStatus,
    pub priority: ServiceOrderPriority,
    pub intake_notes: Option<String>,
    pub intake_at: DateTime<Utc>,
    pub intake_by_user_id: Option<Uuid>,
    pub promised_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub generated_sale_id: Option<Uuid>,
    pub canceled_reason: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub total_amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ServiceOrder> for ServiceOrderResponse {
    fn from(o: &ServiceOrder) -> Self {
        Self {
            id: o.id().into_uuid(),
            store_id: o.store_id(),
            asset_id: o.asset_id().into_uuid(),
            customer_id: o.customer_id(),
            customer_name: o.customer_name().to_string(),
            customer_email: o.customer_email().to_string(),
            customer_phone: o.customer_phone().map(String::from),
            status: o.status(),
            priority: o.priority(),
            intake_notes: o.intake_notes().map(String::from),
            intake_at: o.intake_at(),
            intake_by_user_id: o.intake_by_user_id(),
            promised_at: o.promised_at(),
            delivered_at: o.delivered_at(),
            generated_sale_id: o.generated_sale_id(),
            canceled_reason: o.canceled_reason().map(String::from),
            canceled_at: o.canceled_at(),
            total_amount: o.total_amount(),
            created_at: o.created_at(),
            updated_at: o.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrderItemResponse {
    pub id: Uuid,
    pub service_order_id: Uuid,
    pub item_type: ServiceOrderItemType,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub total: Decimal,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<&ServiceOrderItem> for ServiceOrderItemResponse {
    fn from(i: &ServiceOrderItem) -> Self {
        Self {
            id: i.id().into_uuid(),
            service_order_id: i.service_order_id().into_uuid(),
            item_type: i.item_type(),
            description: i.description().to_string(),
            quantity: i.quantity(),
            unit_price: i.unit_price(),
            total: i.total(),
            product_id: i.product_id(),
            variant_id: i.variant_id(),
            tax_rate: i.tax_rate(),
            tax_amount: i.tax_amount(),
            created_at: i.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResponse {
    pub id: Uuid,
    pub service_order_id: Uuid,
    pub technician_user_id: Option<Uuid>,
    pub findings: String,
    pub recommended_actions: Option<String>,
    pub severity: DiagnosticSeverity,
    pub created_at: DateTime<Utc>,
}

impl From<&Diagnostic> for DiagnosticResponse {
    fn from(d: &Diagnostic) -> Self {
        Self {
            id: d.id().into_uuid(),
            service_order_id: d.service_order_id().into_uuid(),
            technician_user_id: d.technician_user_id(),
            findings: d.findings().to_string(),
            recommended_actions: d.recommended_actions().map(String::from),
            severity: d.severity(),
            created_at: d.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub id: Uuid,
    pub service_order_id: Uuid,
    pub version: i32,
    pub labor_total: Decimal,
    pub parts_total: Decimal,
    pub tax_total: Decimal,
    pub grand_total: Decimal,
    pub valid_until: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub status: QuoteStatus,
    pub sent_at: Option<DateTime<Utc>>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by_customer: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&Quote> for QuoteResponse {
    fn from(q: &Quote) -> Self {
        Self {
            id: q.id().into_uuid(),
            service_order_id: q.service_order_id().into_uuid(),
            version: q.version(),
            labor_total: q.labor_total(),
            parts_total: q.parts_total(),
            tax_total: q.tax_total(),
            grand_total: q.grand_total(),
            valid_until: q.valid_until(),
            notes: q.notes().map(String::from),
            status: q.status(),
            sent_at: q.sent_at(),
            decided_at: q.decided_at(),
            decided_by_customer: q.decided_by_customer(),
            created_at: q.created_at(),
        }
    }
}

/// Detailed view returned by `GET /api/v1/service-orders/{id}` and the public
/// status endpoint — bundles the order with its sub-resources so the client
/// renders without extra round-trips.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrderDetailResponse {
    #[serde(flatten)]
    pub order: ServiceOrderResponse,
    pub items: Vec<ServiceOrderItemResponse>,
    pub diagnostics: Vec<DiagnosticResponse>,
    pub quotes: Vec<QuoteResponse>,
}

/// Public endpoint payload. Same content as the full detail but hides the
/// `public_token` (which the caller already has in the URL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicServiceOrderResponse {
    pub id: Uuid,
    pub status: ServiceOrderStatus,
    pub priority: ServiceOrderPriority,
    pub asset_id: Uuid,
    pub customer_name: String,
    pub intake_at: DateTime<Utc>,
    pub promised_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub total_amount: Decimal,
    pub items: Vec<ServiceOrderItemResponse>,
    pub latest_diagnostic: Option<DiagnosticResponse>,
    pub latest_quote: Option<QuoteResponse>,
}
