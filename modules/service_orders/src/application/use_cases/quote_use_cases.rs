//! Quote workflow. Creating a new quote bumps the version and supersedes any
//! previously open (Draft|Sent) quote on the order. Approving / rejecting a
//! Sent quote also drives the parent `ServiceOrder`'s state machine.

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::ServiceOrdersError;
use crate::application::dtos::{CreateQuoteCommand, DecideQuoteCommand};
use crate::domain::entities::Quote;
use crate::domain::repositories::{
    QuoteRepository, ServiceOrderItemRepository, ServiceOrderRepository,
};
use crate::domain::value_objects::{
    QuoteId, ServiceOrderId, ServiceOrderItemType, ServiceOrderStatus,
};

pub struct CreateQuoteUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    items: Arc<dyn ServiceOrderItemRepository>,
    quotes: Arc<dyn QuoteRepository>,
}

impl CreateQuoteUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        items: Arc<dyn ServiceOrderItemRepository>,
        quotes: Arc<dyn QuoteRepository>,
    ) -> Self {
        Self {
            orders,
            items,
            quotes,
        }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
        cmd: CreateQuoteCommand,
    ) -> Result<Quote, ServiceOrdersError> {
        let order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;

        // Quotes only make sense once the order is past Intake. The order
        // transition to QuoteSent happens at SendQuote time, not here.
        if matches!(
            order.status(),
            ServiceOrderStatus::Intake
                | ServiceOrderStatus::Delivered
                | ServiceOrderStatus::Canceled
        ) {
            return Err(ServiceOrdersError::Validation(format!(
                "cannot draft a quote while order is in status `{}`",
                order.status().as_str()
            )));
        }

        // Compute totals from current items.
        let order_items = self.items.list_by_order(order_id).await?;
        let mut labor_total = Decimal::ZERO;
        let mut parts_total = Decimal::ZERO;
        let mut tax_total = Decimal::ZERO;
        for it in &order_items {
            let subtotal = it.quantity() * it.unit_price();
            match it.item_type() {
                ServiceOrderItemType::Labor => labor_total += subtotal,
                ServiceOrderItemType::Part => parts_total += subtotal,
            }
            tax_total += it.tax_amount();
        }

        let next_version = self.quotes.max_version_for_order(order_id).await? + 1;
        let quote = Quote::draft(
            order_id,
            next_version,
            labor_total.round_dp(4),
            parts_total.round_dp(4),
            tax_total.round_dp(4),
            cmd.valid_until,
            cmd.notes,
        )?;
        self.quotes.save(&quote).await?;
        // Mark any older Draft|Sent quote on the same order as Superseded.
        self.quotes
            .mark_others_superseded(order_id, quote.id())
            .await?;
        Ok(quote)
    }
}

pub struct SendQuoteUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    quotes: Arc<dyn QuoteRepository>,
}

impl SendQuoteUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>, quotes: Arc<dyn QuoteRepository>) -> Self {
        Self { orders, quotes }
    }

    pub async fn execute(&self, quote_id: QuoteId) -> Result<Quote, ServiceOrdersError> {
        let mut quote = self
            .quotes
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::QuoteNotFound(quote_id.into_uuid()))?;
        let order_id = quote.service_order_id();
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;

        quote.send()?;
        // Move order to QuoteSent if it isn't already there (idempotent on
        // resends — the second send keeps order in QuoteSent).
        if order.status() == ServiceOrderStatus::Diagnosis {
            order.submit_quote()?;
            self.orders.update(&order).await?;
        }
        self.quotes.update(&quote).await?;
        Ok(quote)
    }
}

pub struct ApproveQuoteUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    quotes: Arc<dyn QuoteRepository>,
}

impl ApproveQuoteUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>, quotes: Arc<dyn QuoteRepository>) -> Self {
        Self { orders, quotes }
    }

    pub async fn execute(
        &self,
        quote_id: QuoteId,
        cmd: DecideQuoteCommand,
    ) -> Result<Quote, ServiceOrdersError> {
        let mut quote = self
            .quotes
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::QuoteNotFound(quote_id.into_uuid()))?;
        let order_id = quote.service_order_id();
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;

        quote.approve(cmd.decided_by_customer)?;
        order.approve_quote()?;
        self.quotes.update(&quote).await?;
        self.orders.update(&order).await?;
        Ok(quote)
    }
}

pub struct RejectQuoteUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    quotes: Arc<dyn QuoteRepository>,
}

impl RejectQuoteUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>, quotes: Arc<dyn QuoteRepository>) -> Self {
        Self { orders, quotes }
    }

    pub async fn execute(
        &self,
        quote_id: QuoteId,
        cmd: DecideQuoteCommand,
    ) -> Result<Quote, ServiceOrdersError> {
        let mut quote = self
            .quotes
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::QuoteNotFound(quote_id.into_uuid()))?;
        let order_id = quote.service_order_id();
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;

        quote.reject(cmd.decided_by_customer)?;
        // Order goes back to Diagnosis so a new quote can be drafted.
        order.reject_quote()?;
        self.quotes.update(&quote).await?;
        self.orders.update(&order).await?;
        Ok(quote)
    }
}

pub struct ListQuotesUseCase {
    quotes: Arc<dyn QuoteRepository>,
}

impl ListQuotesUseCase {
    pub fn new(quotes: Arc<dyn QuoteRepository>) -> Self {
        Self { quotes }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Quote>, ServiceOrdersError> {
        self.quotes.list_by_order(order_id).await
    }
}
