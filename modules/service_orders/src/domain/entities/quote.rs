//! Quote — versioned cost estimate. The repository's
//! `mark_others_superseded` keeps at most one active (Draft|Sent) quote per
//! order at any time; older quotes are kept for audit but flagged
//! `Superseded`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;
use crate::domain::value_objects::{QuoteId, QuoteStatus, ServiceOrderId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    id: QuoteId,
    service_order_id: ServiceOrderId,
    version: i32,
    labor_total: Decimal,
    parts_total: Decimal,
    tax_total: Decimal,
    grand_total: Decimal,
    valid_until: Option<DateTime<Utc>>,
    notes: Option<String>,
    status: QuoteStatus,
    sent_at: Option<DateTime<Utc>>,
    decided_at: Option<DateTime<Utc>>,
    decided_by_customer: bool,
    created_at: DateTime<Utc>,
}

impl Quote {
    #[allow(clippy::too_many_arguments)]
    pub fn draft(
        service_order_id: ServiceOrderId,
        version: i32,
        labor_total: Decimal,
        parts_total: Decimal,
        tax_total: Decimal,
        valid_until: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Result<Self, ServiceOrdersError> {
        if version < 1 {
            return Err(ServiceOrdersError::Validation(
                "quote version must be >= 1".to_string(),
            ));
        }
        if labor_total < Decimal::ZERO || parts_total < Decimal::ZERO || tax_total < Decimal::ZERO {
            return Err(ServiceOrdersError::Validation(
                "quote totals cannot be negative".to_string(),
            ));
        }
        let grand_total = labor_total + parts_total + tax_total;
        Ok(Self {
            id: QuoteId::new(),
            service_order_id,
            version,
            labor_total,
            parts_total,
            tax_total,
            grand_total,
            valid_until,
            notes,
            status: QuoteStatus::Draft,
            sent_at: None,
            decided_at: None,
            decided_by_customer: false,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: QuoteId,
        service_order_id: ServiceOrderId,
        version: i32,
        labor_total: Decimal,
        parts_total: Decimal,
        tax_total: Decimal,
        grand_total: Decimal,
        valid_until: Option<DateTime<Utc>>,
        notes: Option<String>,
        status: QuoteStatus,
        sent_at: Option<DateTime<Utc>>,
        decided_at: Option<DateTime<Utc>>,
        decided_by_customer: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            service_order_id,
            version,
            labor_total,
            parts_total,
            tax_total,
            grand_total,
            valid_until,
            notes,
            status,
            sent_at,
            decided_at,
            decided_by_customer,
            created_at,
        }
    }

    fn transition(&mut self, to: QuoteStatus) -> Result<(), ServiceOrdersError> {
        if !self.status.can_transition_to(to) {
            return Err(ServiceOrdersError::InvalidQuoteStateTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        Ok(())
    }

    pub fn send(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(QuoteStatus::Sent)?;
        self.sent_at = Some(Utc::now());
        Ok(())
    }

    pub fn approve(&mut self, decided_by_customer: bool) -> Result<(), ServiceOrdersError> {
        self.transition(QuoteStatus::Approved)?;
        self.decided_at = Some(Utc::now());
        self.decided_by_customer = decided_by_customer;
        Ok(())
    }

    pub fn reject(&mut self, decided_by_customer: bool) -> Result<(), ServiceOrdersError> {
        self.transition(QuoteStatus::Rejected)?;
        self.decided_at = Some(Utc::now());
        self.decided_by_customer = decided_by_customer;
        Ok(())
    }

    pub fn supersede(&mut self) -> Result<(), ServiceOrdersError> {
        self.transition(QuoteStatus::Superseded)
    }

    pub fn id(&self) -> QuoteId {
        self.id
    }
    pub fn service_order_id(&self) -> ServiceOrderId {
        self.service_order_id
    }
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn labor_total(&self) -> Decimal {
        self.labor_total
    }
    pub fn parts_total(&self) -> Decimal {
        self.parts_total
    }
    pub fn tax_total(&self) -> Decimal {
        self.tax_total
    }
    pub fn grand_total(&self) -> Decimal {
        self.grand_total
    }
    pub fn valid_until(&self) -> Option<DateTime<Utc>> {
        self.valid_until
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn status(&self) -> QuoteStatus {
        self.status
    }
    pub fn sent_at(&self) -> Option<DateTime<Utc>> {
        self.sent_at
    }
    pub fn decided_at(&self) -> Option<DateTime<Utc>> {
        self.decided_at
    }
    pub fn decided_by_customer(&self) -> bool {
        self.decided_by_customer
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
