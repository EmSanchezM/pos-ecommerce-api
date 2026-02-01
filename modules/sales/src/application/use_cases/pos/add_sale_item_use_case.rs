//! Add sale item use case

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::application::dtos::{AddSaleItemCommand, SaleDetailResponse};
use crate::domain::entities::SaleItem;
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleId;
use crate::SalesError;
use inventory::{ProductId, UnitOfMeasure, VariantId};

/// Use case for adding an item to a sale
pub struct AddSaleItemUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl AddSaleItemUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(
        &self,
        cmd: AddSaleItemCommand,
        sku: String,
        description: String,
        unit_price: Decimal,
        unit_cost: Decimal,
        tax_rate: Decimal,
        unit_of_measure: UnitOfMeasure,
    ) -> Result<SaleDetailResponse, SalesError> {
        let sale_id = SaleId::from_uuid(cmd.sale_id);

        let mut sale = self
            .sale_repo
            .find_by_id_with_details(sale_id)
            .await?
            .ok_or(SalesError::SaleNotFound(cmd.sale_id))?;

        // Verify sale is editable
        if !sale.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }

        let line_number = sale.item_count() as i32 + 1;
        let final_price = cmd.unit_price.unwrap_or(unit_price);

        // Create the sale item
        let item = SaleItem::create(
            sale_id,
            line_number,
            ProductId::from_uuid(cmd.product_id),
            cmd.variant_id.map(VariantId::from_uuid),
            sku,
            description,
            cmd.quantity,
            unit_of_measure,
            final_price,
            unit_cost,
            tax_rate,
        )?;

        // Add item to sale and recalculate totals
        sale.add_item(item.clone())?;

        // Save the item
        self.sale_repo.save_item(&item).await?;

        // Update sale totals
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
