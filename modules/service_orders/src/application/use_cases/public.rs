//! Public, unauthenticated status lookup. The customer hits
//! `/api/v1/public/service-orders/{id}?token=...` to see where their repair
//! is. We return a curated payload (no `public_token`, no internal user ids
//! beyond what the customer already knows about).

use std::sync::Arc;

use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::application::dtos::{
    DiagnosticResponse, PublicServiceOrderResponse, QuoteResponse, ServiceOrderItemResponse,
};
use crate::domain::repositories::{
    DiagnosticRepository, QuoteRepository, ServiceOrderItemRepository, ServiceOrderRepository,
};

pub struct GetPublicServiceOrderUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    items: Arc<dyn ServiceOrderItemRepository>,
    diagnostics: Arc<dyn DiagnosticRepository>,
    quotes: Arc<dyn QuoteRepository>,
}

impl GetPublicServiceOrderUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        items: Arc<dyn ServiceOrderItemRepository>,
        diagnostics: Arc<dyn DiagnosticRepository>,
        quotes: Arc<dyn QuoteRepository>,
    ) -> Self {
        Self {
            orders,
            items,
            diagnostics,
            quotes,
        }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        token: &str,
    ) -> Result<PublicServiceOrderResponse, ServiceOrdersError> {
        let order = self
            .orders
            .find_by_public_token(token)
            .await?
            .ok_or(ServiceOrdersError::InvalidPublicToken)?;
        if order.id().into_uuid() != id {
            return Err(ServiceOrdersError::InvalidPublicToken);
        }

        let order_id = order.id();
        let items = self.items.list_by_order(order_id).await?;
        let diagnostics = self.diagnostics.list_by_order(order_id).await?;
        let quotes = self.quotes.list_by_order(order_id).await?;

        // Latest = most recent by created_at; the repository returns ordered
        // lists, but defensive sort here keeps the contract obvious.
        let latest_diagnostic = diagnostics
            .iter()
            .max_by_key(|d| d.created_at())
            .map(DiagnosticResponse::from);
        let latest_quote = quotes
            .iter()
            .max_by_key(|q| q.created_at())
            .map(QuoteResponse::from);

        Ok(PublicServiceOrderResponse {
            id: order.id().into_uuid(),
            status: order.status(),
            priority: order.priority(),
            asset_id: order.asset_id().into_uuid(),
            customer_name: order.customer_name().to_string(),
            intake_at: order.intake_at(),
            promised_at: order.promised_at(),
            delivered_at: order.delivered_at(),
            total_amount: order.total_amount(),
            items: items.iter().map(ServiceOrderItemResponse::from).collect(),
            latest_diagnostic,
            latest_quote,
        })
    }
}
