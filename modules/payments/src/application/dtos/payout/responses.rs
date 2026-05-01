//! Payout response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::Payout;

#[derive(Debug, Serialize)]
pub struct PayoutResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub gateway_id: Uuid,
    pub status: String,
    pub amount: Decimal,
    pub currency: String,
    pub fee_amount: Decimal,
    pub net_amount: Decimal,
    pub gateway_payout_id: Option<String>,
    pub transaction_count: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub expected_arrival: Option<DateTime<Utc>>,
    pub arrived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Payout> for PayoutResponse {
    fn from(p: Payout) -> Self {
        Self {
            id: p.id().into_uuid(),
            store_id: p.store_id().into_uuid(),
            gateway_id: p.gateway_id().into_uuid(),
            status: p.status().to_string(),
            amount: p.amount(),
            currency: p.currency().to_string(),
            fee_amount: p.fee_amount(),
            net_amount: p.net_amount(),
            gateway_payout_id: p.gateway_payout_id().map(str::to_string),
            transaction_count: p.transaction_count(),
            period_start: p.period_start(),
            period_end: p.period_end(),
            expected_arrival: p.expected_arrival(),
            arrived_at: p.arrived_at(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PayoutListResponse {
    pub items: Vec<PayoutResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
