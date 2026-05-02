use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertReorderPolicyCommand {
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub lead_time_days: i32,
    #[serde(default)]
    pub safety_stock_qty: Decimal,
    pub review_cycle_days: i32,
    pub preferred_vendor_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveSuggestionCommand {
    /// Date the generated PO will use as `order_date` (YYYY-MM-DD).
    pub order_date: String,
    /// Optional override: unit cost for the PO line.
    pub unit_cost: Decimal,
    /// Required: PurchaseOrderItem expects a UoM string like "unit", "kg".
    pub unit_of_measure: String,
    /// Optional: vendor to use if the suggestion has none.
    pub vendor_id: Option<Uuid>,
    /// Optional: free-form description for the PO line.
    pub line_description: Option<String>,
    /// Required: product the variant belongs to (PO items reference both).
    pub product_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissSuggestionCommand {
    pub reason: String,
}
