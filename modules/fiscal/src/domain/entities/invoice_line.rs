//! InvoiceLine entity - represents a line item in a fiscal invoice

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{InvoiceId, InvoiceLineId, TaxType};

/// InvoiceLine entity representing a line item in a fiscal invoice.
///
/// Invariants:
/// - Quantity must be positive
/// - Unit price must be non-negative
/// - Line number must be positive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    id: InvoiceLineId,
    invoice_id: InvoiceId,
    line_number: i32,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    sku: String,
    description: String,
    quantity: Decimal,
    unit_of_measure: String,
    unit_price: Decimal,
    discount_amount: Decimal,
    tax_type: TaxType,
    tax_rate: Decimal,
    tax_amount: Decimal,
    subtotal: Decimal,
    total: Decimal,
    is_exempt: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl InvoiceLine {
    /// Creates a new InvoiceLine
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        invoice_id: InvoiceId,
        line_number: i32,
        product_id: uuid::Uuid,
        variant_id: Option<uuid::Uuid>,
        sku: String,
        description: String,
        quantity: Decimal,
        unit_of_measure: String,
        unit_price: Decimal,
        discount_amount: Decimal,
        tax_type: TaxType,
        tax_rate: Decimal,
        tax_amount: Decimal,
        subtotal: Decimal,
        total: Decimal,
        is_exempt: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: InvoiceLineId::new(),
            invoice_id,
            line_number,
            product_id,
            variant_id,
            sku,
            description,
            quantity,
            unit_of_measure,
            unit_price,
            discount_amount,
            tax_type,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            is_exempt,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes an InvoiceLine from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: InvoiceLineId,
        invoice_id: InvoiceId,
        line_number: i32,
        product_id: uuid::Uuid,
        variant_id: Option<uuid::Uuid>,
        sku: String,
        description: String,
        quantity: Decimal,
        unit_of_measure: String,
        unit_price: Decimal,
        discount_amount: Decimal,
        tax_type: TaxType,
        tax_rate: Decimal,
        tax_amount: Decimal,
        subtotal: Decimal,
        total: Decimal,
        is_exempt: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            invoice_id,
            line_number,
            product_id,
            variant_id,
            sku,
            description,
            quantity,
            unit_of_measure,
            unit_price,
            discount_amount,
            tax_type,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            is_exempt,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> InvoiceLineId {
        self.id
    }

    pub fn invoice_id(&self) -> InvoiceId {
        self.invoice_id
    }

    pub fn line_number(&self) -> i32 {
        self.line_number
    }

    pub fn product_id(&self) -> uuid::Uuid {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<uuid::Uuid> {
        self.variant_id
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn unit_of_measure(&self) -> &str {
        &self.unit_of_measure
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn discount_amount(&self) -> Decimal {
        self.discount_amount
    }

    pub fn tax_type(&self) -> TaxType {
        self.tax_type
    }

    pub fn tax_rate(&self) -> Decimal {
        self.tax_rate
    }

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn is_exempt(&self) -> bool {
        self.is_exempt
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
