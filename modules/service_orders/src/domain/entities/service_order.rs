//! ServiceOrder — the workshop ticket aggregate root. Owns the long workflow:
//!
//! ```text
//! Intake ──diagnose──▶ Diagnosis ──submit_quote──▶ QuoteSent
//!                          ▲                          │
//!                          └──reject_quote────────────┤
//!                                                      │
//!                                                approve_quote
//!                                                      ▼
//! QuoteApproved ──start_repair──▶ InRepair ──start_testing──▶ Testing
//!                                                              │
//!                                                          mark_ready
//!                                                              ▼
//!                                                      ReadyForPickup ──deliver──▶ Delivered
//!
//! cancel: any non-terminal → Canceled
//! ```
//!
//! `customer_id` is optional (some intakes are walk-ins without a registered
//! Customer); `customer_name` / `customer_email` snapshots cover that path.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::ServiceOrdersError;
use crate::domain::value_objects::{
    AssetId, ServiceOrderId, ServiceOrderPriority, ServiceOrderStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrder {
    id: ServiceOrderId,
    store_id: Uuid,
    asset_id: AssetId,
    customer_id: Option<Uuid>,
    customer_name: String,
    customer_email: String,
    customer_phone: Option<String>,
    status: ServiceOrderStatus,
    priority: ServiceOrderPriority,
    intake_notes: Option<String>,
    intake_at: DateTime<Utc>,
    intake_by_user_id: Option<Uuid>,
    promised_at: Option<DateTime<Utc>>,
    delivered_at: Option<DateTime<Utc>>,
    generated_sale_id: Option<Uuid>,
    canceled_reason: Option<String>,
    canceled_at: Option<DateTime<Utc>>,
    public_token: String,
    total_amount: Decimal,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ServiceOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn intake(
        store_id: Uuid,
        asset_id: AssetId,
        customer_id: Option<Uuid>,
        customer_name: String,
        customer_email: String,
        customer_phone: Option<String>,
        priority: ServiceOrderPriority,
        intake_notes: Option<String>,
        intake_by_user_id: Option<Uuid>,
        promised_at: Option<DateTime<Utc>>,
    ) -> Result<Self, ServiceOrdersError> {
        if customer_name.trim().is_empty() || customer_email.trim().is_empty() {
            return Err(ServiceOrdersError::Validation(
                "customer_name and customer_email are required".to_string(),
            ));
        }
        let now = Utc::now();
        let public_token = Uuid::new_v7(Timestamp::now(NoContext)).simple().to_string();
        Ok(Self {
            id: ServiceOrderId::new(),
            store_id,
            asset_id,
            customer_id,
            customer_name,
            customer_email,
            customer_phone,
            status: ServiceOrderStatus::Intake,
            priority,
            intake_notes,
            intake_at: now,
            intake_by_user_id,
            promised_at,
            delivered_at: None,
            generated_sale_id: None,
            canceled_reason: None,
            canceled_at: None,
            public_token,
            total_amount: Decimal::ZERO,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ServiceOrderId,
        store_id: Uuid,
        asset_id: AssetId,
        customer_id: Option<Uuid>,
        customer_name: String,
        customer_email: String,
        customer_phone: Option<String>,
        status: ServiceOrderStatus,
        priority: ServiceOrderPriority,
        intake_notes: Option<String>,
        intake_at: DateTime<Utc>,
        intake_by_user_id: Option<Uuid>,
        promised_at: Option<DateTime<Utc>>,
        delivered_at: Option<DateTime<Utc>>,
        generated_sale_id: Option<Uuid>,
        canceled_reason: Option<String>,
        canceled_at: Option<DateTime<Utc>>,
        public_token: String,
        total_amount: Decimal,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            asset_id,
            customer_id,
            customer_name,
            customer_email,
            customer_phone,
            status,
            priority,
            intake_notes,
            intake_at,
            intake_by_user_id,
            promised_at,
            delivered_at,
            generated_sale_id,
            canceled_reason,
            canceled_at,
            public_token,
            total_amount,
            created_at,
            updated_at,
        }
    }

    fn transition(&mut self, to: ServiceOrderStatus) -> Result<(), ServiceOrdersError> {
        if !self.status.can_transition_to(to) {
            return Err(ServiceOrdersError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn diagnose(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::Diagnosis)
    }

    /// Marks the order as `QuoteSent`. Called by the quote use case after the
    /// quote row is persisted.
    pub fn submit_quote(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::QuoteSent)
    }

    pub fn approve_quote(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::QuoteApproved)
    }

    /// Customer rejected the quote — order goes back to Diagnosis so a new
    /// quote can be drafted.
    pub fn reject_quote(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::Diagnosis)
    }

    pub fn start_repair(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::InRepair)
    }

    pub fn start_testing(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::Testing)
    }

    pub fn mark_ready(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::ReadyForPickup)
    }

    pub fn deliver(&mut self, generated_sale_id: Option<Uuid>) -> Result<(), ServiceOrdersError> {
        self.transition(ServiceOrderStatus::Delivered)?;
        self.delivered_at = Some(Utc::now());
        self.generated_sale_id = generated_sale_id;
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), ServiceOrdersError> {
        if reason.trim().is_empty() {
            return Err(ServiceOrdersError::Validation(
                "cancel reason is required".to_string(),
            ));
        }
        self.transition(ServiceOrderStatus::Canceled)?;
        self.canceled_reason = Some(reason);
        self.canceled_at = Some(Utc::now());
        Ok(())
    }

    pub fn recompute_total(&mut self, total: Decimal) {
        self.total_amount = total;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ServiceOrderId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
    pub fn customer_id(&self) -> Option<Uuid> {
        self.customer_id
    }
    pub fn customer_name(&self) -> &str {
        &self.customer_name
    }
    pub fn customer_email(&self) -> &str {
        &self.customer_email
    }
    pub fn customer_phone(&self) -> Option<&str> {
        self.customer_phone.as_deref()
    }
    pub fn status(&self) -> ServiceOrderStatus {
        self.status
    }
    pub fn priority(&self) -> ServiceOrderPriority {
        self.priority
    }
    pub fn intake_notes(&self) -> Option<&str> {
        self.intake_notes.as_deref()
    }
    pub fn intake_at(&self) -> DateTime<Utc> {
        self.intake_at
    }
    pub fn intake_by_user_id(&self) -> Option<Uuid> {
        self.intake_by_user_id
    }
    pub fn promised_at(&self) -> Option<DateTime<Utc>> {
        self.promised_at
    }
    pub fn delivered_at(&self) -> Option<DateTime<Utc>> {
        self.delivered_at
    }
    pub fn generated_sale_id(&self) -> Option<Uuid> {
        self.generated_sale_id
    }
    pub fn canceled_reason(&self) -> Option<&str> {
        self.canceled_reason.as_deref()
    }
    pub fn canceled_at(&self) -> Option<DateTime<Utc>> {
        self.canceled_at
    }
    pub fn public_token(&self) -> &str {
        &self.public_token
    }
    pub fn total_amount(&self) -> Decimal {
        self.total_amount
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
