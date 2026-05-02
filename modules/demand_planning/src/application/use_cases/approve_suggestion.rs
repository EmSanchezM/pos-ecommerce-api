//! ApproveSuggestionUseCase — moves a `pending` suggestion to `approved` and
//! creates a draft Purchase Order in the `purchasing` module with the
//! suggested vendor and quantity. The newly created PO id is stored back on
//! the suggestion which transitions to `ordered`.
//!
//! Note: the PO is created in the `draft` status set by
//! `purchasing::CreatePurchaseOrderUseCase`. Submission/approval of the PO
//! itself remains the buyer's responsibility — the demand planning module is
//! intentionally hands-off after this point.

use std::sync::Arc;

use identity::UserId;
use purchasing::{
    CreatePurchaseOrderCommand, CreatePurchaseOrderItemCommand, CreatePurchaseOrderUseCase,
    PgPurchaseOrderRepository, PgVendorRepository,
};
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::application::dtos::ApproveSuggestionCommand;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::repositories::ReplenishmentSuggestionRepository;
use crate::domain::value_objects::SuggestionId;

pub struct ApproveSuggestionUseCase {
    suggestions: Arc<dyn ReplenishmentSuggestionRepository>,
    create_po: Arc<CreatePurchaseOrderUseCase<PgPurchaseOrderRepository, PgVendorRepository>>,
}

impl ApproveSuggestionUseCase {
    pub fn new(
        suggestions: Arc<dyn ReplenishmentSuggestionRepository>,
        create_po: Arc<CreatePurchaseOrderUseCase<PgPurchaseOrderRepository, PgVendorRepository>>,
    ) -> Self {
        Self {
            suggestions,
            create_po,
        }
    }

    pub async fn execute(
        &self,
        id: SuggestionId,
        actor_id: Uuid,
        cmd: ApproveSuggestionCommand,
    ) -> Result<ReplenishmentSuggestion, DemandPlanningError> {
        let mut suggestion = self
            .suggestions
            .find_by_id(id)
            .await?
            .ok_or_else(|| DemandPlanningError::SuggestionNotFound(id.into_uuid()))?;

        suggestion.approve(actor_id)?;

        let vendor_id = cmd
            .vendor_id
            .or_else(|| suggestion.suggested_vendor_id())
            .ok_or_else(|| {
                DemandPlanningError::Subscriber(
                    "Approve requires a vendor_id (no preferred vendor on the suggestion)".into(),
                )
            })?;

        // demand_planning's `product_variant_id` collapses (variant_id, product_id)
        // into one column via COALESCE, so a "variantless" product is identified
        // by its own product_id. The PO `variant_id` column is a real FK to
        // `product_variants` though, so we only forward it when there's an
        // actual variant — otherwise the FK insert fails.
        let variant_id = if suggestion.product_variant_id() == cmd.product_id {
            None
        } else {
            Some(suggestion.product_variant_id())
        };
        let item = CreatePurchaseOrderItemCommand {
            product_id: cmd.product_id,
            variant_id,
            description: cmd
                .line_description
                .unwrap_or_else(|| "Replenishment suggestion".into()),
            quantity_ordered: suggestion.recommended_qty(),
            unit_of_measure: cmd.unit_of_measure,
            unit_cost: cmd.unit_cost,
            discount_percent: rust_decimal::Decimal::ZERO,
            tax_percent: rust_decimal::Decimal::ZERO,
            notes: None,
        };
        let po_cmd = CreatePurchaseOrderCommand {
            store_id: suggestion.store_id(),
            vendor_id,
            order_date: cmd.order_date,
            expected_delivery_date: None,
            currency: None,
            payment_terms_days: None,
            notes: Some(format!(
                "Auto-generated from replenishment suggestion {}",
                id.into_uuid()
            )),
            items: vec![item],
        };

        let po = self
            .create_po
            .execute(po_cmd, UserId::from_uuid(actor_id))
            .await
            .map_err(|e| DemandPlanningError::Subscriber(format!("purchasing: {}", e)))?;

        suggestion.mark_ordered(po.id)?;
        self.suggestions.update(&suggestion).await?;
        Ok(suggestion)
    }
}
