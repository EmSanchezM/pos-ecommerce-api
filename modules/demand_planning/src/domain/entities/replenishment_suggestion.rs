//! ReplenishmentSuggestion entity — a recommendation produced by the
//! `GenerateReplenishmentSuggestionsUseCase`. The aggregate enforces the
//! `pending → approved → ordered` and `pending → dismissed` state machine.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::value_objects::{SuggestionId, SuggestionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplenishmentSuggestion {
    id: SuggestionId,
    product_variant_id: Uuid,
    store_id: Uuid,
    current_stock: Decimal,
    forecast_qty: Decimal,
    recommended_qty: Decimal,
    suggested_vendor_id: Option<Uuid>,
    status: SuggestionStatus,
    generated_at: DateTime<Utc>,
    decided_at: Option<DateTime<Utc>>,
    decided_by: Option<Uuid>,
    generated_purchase_order_id: Option<Uuid>,
    dismiss_reason: Option<String>,
}

impl ReplenishmentSuggestion {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        product_variant_id: Uuid,
        store_id: Uuid,
        current_stock: Decimal,
        forecast_qty: Decimal,
        recommended_qty: Decimal,
        suggested_vendor_id: Option<Uuid>,
    ) -> Result<Self, DemandPlanningError> {
        if recommended_qty < Decimal::ZERO || current_stock < Decimal::ZERO {
            return Err(DemandPlanningError::NegativeQuantity);
        }
        Ok(Self {
            id: SuggestionId::new(),
            product_variant_id,
            store_id,
            current_stock,
            forecast_qty,
            recommended_qty,
            suggested_vendor_id,
            status: SuggestionStatus::Pending,
            generated_at: Utc::now(),
            decided_at: None,
            decided_by: None,
            generated_purchase_order_id: None,
            dismiss_reason: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SuggestionId,
        product_variant_id: Uuid,
        store_id: Uuid,
        current_stock: Decimal,
        forecast_qty: Decimal,
        recommended_qty: Decimal,
        suggested_vendor_id: Option<Uuid>,
        status: SuggestionStatus,
        generated_at: DateTime<Utc>,
        decided_at: Option<DateTime<Utc>>,
        decided_by: Option<Uuid>,
        generated_purchase_order_id: Option<Uuid>,
        dismiss_reason: Option<String>,
    ) -> Self {
        Self {
            id,
            product_variant_id,
            store_id,
            current_stock,
            forecast_qty,
            recommended_qty,
            suggested_vendor_id,
            status,
            generated_at,
            decided_at,
            decided_by,
            generated_purchase_order_id,
            dismiss_reason,
        }
    }

    pub fn approve(&mut self, decided_by: Uuid) -> Result<(), DemandPlanningError> {
        if !self.status.can_transition_to(SuggestionStatus::Approved) {
            return Err(DemandPlanningError::InvalidSuggestionTransition {
                from: self.status.to_string(),
                to: SuggestionStatus::Approved.to_string(),
            });
        }
        self.status = SuggestionStatus::Approved;
        self.decided_at = Some(Utc::now());
        self.decided_by = Some(decided_by);
        Ok(())
    }

    pub fn mark_ordered(&mut self, purchase_order_id: Uuid) -> Result<(), DemandPlanningError> {
        if !self.status.can_transition_to(SuggestionStatus::Ordered) {
            return Err(DemandPlanningError::InvalidSuggestionTransition {
                from: self.status.to_string(),
                to: SuggestionStatus::Ordered.to_string(),
            });
        }
        self.status = SuggestionStatus::Ordered;
        self.generated_purchase_order_id = Some(purchase_order_id);
        Ok(())
    }

    pub fn dismiss(&mut self, decided_by: Uuid, reason: String) -> Result<(), DemandPlanningError> {
        if reason.trim().is_empty() {
            return Err(DemandPlanningError::DismissReasonRequired);
        }
        if !self.status.can_transition_to(SuggestionStatus::Dismissed) {
            return Err(DemandPlanningError::InvalidSuggestionTransition {
                from: self.status.to_string(),
                to: SuggestionStatus::Dismissed.to_string(),
            });
        }
        self.status = SuggestionStatus::Dismissed;
        self.decided_at = Some(Utc::now());
        self.decided_by = Some(decided_by);
        self.dismiss_reason = Some(reason);
        Ok(())
    }

    pub fn id(&self) -> SuggestionId {
        self.id
    }
    pub fn product_variant_id(&self) -> Uuid {
        self.product_variant_id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn current_stock(&self) -> Decimal {
        self.current_stock
    }
    pub fn forecast_qty(&self) -> Decimal {
        self.forecast_qty
    }
    pub fn recommended_qty(&self) -> Decimal {
        self.recommended_qty
    }
    pub fn suggested_vendor_id(&self) -> Option<Uuid> {
        self.suggested_vendor_id
    }
    pub fn status(&self) -> SuggestionStatus {
        self.status
    }
    pub fn generated_at(&self) -> DateTime<Utc> {
        self.generated_at
    }
    pub fn decided_at(&self) -> Option<DateTime<Utc>> {
        self.decided_at
    }
    pub fn decided_by(&self) -> Option<Uuid> {
        self.decided_by
    }
    pub fn generated_purchase_order_id(&self) -> Option<Uuid> {
        self.generated_purchase_order_id
    }
    pub fn dismiss_reason(&self) -> Option<&str> {
        self.dismiss_reason.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn fresh() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    fn build() -> ReplenishmentSuggestion {
        ReplenishmentSuggestion::create(fresh(), fresh(), dec!(5), dec!(20), dec!(45), None)
            .unwrap()
    }

    #[test]
    fn approve_then_order_succeeds() {
        let mut s = build();
        s.approve(fresh()).unwrap();
        s.mark_ordered(fresh()).unwrap();
        assert_eq!(s.status(), SuggestionStatus::Ordered);
        assert!(s.generated_purchase_order_id().is_some());
    }

    #[test]
    fn dismiss_requires_reason() {
        let mut s = build();
        assert!(s.dismiss(fresh(), "  ".into()).is_err());
        assert!(s.dismiss(fresh(), "obsolete sku".into()).is_ok());
        assert_eq!(s.status(), SuggestionStatus::Dismissed);
    }

    #[test]
    fn cannot_approve_after_dismissed() {
        let mut s = build();
        s.dismiss(fresh(), "x".into()).unwrap();
        assert!(s.approve(fresh()).is_err());
    }
}
