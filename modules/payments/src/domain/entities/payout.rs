//! Payout entity - settlement record from a payment gateway.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{PaymentGatewayId, PayoutId, PayoutStatus};
use identity::StoreId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payout {
    id: PayoutId,
    store_id: StoreId,
    gateway_id: PaymentGatewayId,
    status: PayoutStatus,
    amount: Decimal,
    currency: String,
    fee_amount: Decimal,
    net_amount: Decimal,
    gateway_payout_id: Option<String>,
    transaction_count: i32,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    expected_arrival: Option<DateTime<Utc>>,
    arrived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Payout {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        gateway_id: PaymentGatewayId,
        amount: Decimal,
        currency: String,
        fee_amount: Decimal,
        net_amount: Decimal,
        gateway_payout_id: Option<String>,
        transaction_count: i32,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        expected_arrival: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PayoutId::new(),
            store_id,
            gateway_id,
            status: PayoutStatus::Pending,
            amount,
            currency,
            fee_amount,
            net_amount,
            gateway_payout_id,
            transaction_count,
            period_start,
            period_end,
            expected_arrival,
            arrived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PayoutId,
        store_id: StoreId,
        gateway_id: PaymentGatewayId,
        status: PayoutStatus,
        amount: Decimal,
        currency: String,
        fee_amount: Decimal,
        net_amount: Decimal,
        gateway_payout_id: Option<String>,
        transaction_count: i32,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        expected_arrival: Option<DateTime<Utc>>,
        arrived_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            gateway_id,
            status,
            amount,
            currency,
            fee_amount,
            net_amount,
            gateway_payout_id,
            transaction_count,
            period_start,
            period_end,
            expected_arrival,
            arrived_at,
            created_at,
            updated_at,
        }
    }

    pub fn mark_in_transit(&mut self) {
        self.status = PayoutStatus::InTransit;
        self.touch();
    }

    pub fn mark_paid(&mut self, arrived_at: DateTime<Utc>) {
        self.status = PayoutStatus::Paid;
        self.arrived_at = Some(arrived_at);
        self.touch();
    }

    pub fn mark_failed(&mut self) {
        self.status = PayoutStatus::Failed;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    pub fn id(&self) -> PayoutId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn gateway_id(&self) -> PaymentGatewayId {
        self.gateway_id
    }
    pub fn status(&self) -> PayoutStatus {
        self.status
    }
    pub fn amount(&self) -> Decimal {
        self.amount
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn fee_amount(&self) -> Decimal {
        self.fee_amount
    }
    pub fn net_amount(&self) -> Decimal {
        self.net_amount
    }
    pub fn gateway_payout_id(&self) -> Option<&str> {
        self.gateway_payout_id.as_deref()
    }
    pub fn transaction_count(&self) -> i32 {
        self.transaction_count
    }
    pub fn period_start(&self) -> DateTime<Utc> {
        self.period_start
    }
    pub fn period_end(&self) -> DateTime<Utc> {
        self.period_end
    }
    pub fn expected_arrival(&self) -> Option<DateTime<Utc>> {
        self.expected_arrival
    }
    pub fn arrived_at(&self) -> Option<DateTime<Utc>> {
        self.arrived_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
