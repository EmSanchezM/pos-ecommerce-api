use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::value_objects::{
    AssetType, DiagnosticSeverity, ServiceOrderItemType, ServiceOrderPriority,
};

// -----------------------------------------------------------------------------
// Assets
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAssetCommand {
    pub store_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub asset_type: AssetType,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub identifier: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssetCommand {
    pub brand: Option<String>,
    pub model: Option<String>,
    pub identifier: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Option<JsonValue>,
}

// -----------------------------------------------------------------------------
// Service order intake
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeServiceOrderCommand {
    pub store_id: Uuid,
    pub asset_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    #[serde(default)]
    pub priority: Option<ServiceOrderPriority>,
    pub intake_notes: Option<String>,
    pub promised_at: Option<DateTime<Utc>>,
}

// -----------------------------------------------------------------------------
// Items
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemCommand {
    pub item_type: ServiceOrderItemType,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    #[serde(default)]
    pub tax_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItemCommand {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    #[serde(default)]
    pub tax_rate: Option<Decimal>,
}

// -----------------------------------------------------------------------------
// Diagnostics
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDiagnosticCommand {
    pub findings: String,
    pub recommended_actions: Option<String>,
    pub severity: DiagnosticSeverity,
}

// -----------------------------------------------------------------------------
// Quotes
// -----------------------------------------------------------------------------

/// Notes/validity for a new quote. Totals are recomputed from the order's
/// items inside the use case (labor vs parts vs tax).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateQuoteCommand {
    pub valid_until: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecideQuoteCommand {
    /// `true` when the customer was the one to make the call (typical),
    /// `false` when the staff overrode the decision (rare; logged for audit).
    #[serde(default = "default_true")]
    pub decided_by_customer: bool,
}

fn default_true() -> bool {
    true
}

// -----------------------------------------------------------------------------
// Cancellation
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelServiceOrderCommand {
    pub reason: String,
}
