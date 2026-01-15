// ProductVariant entity - product variations (size, color, flavor, etc.)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{Barcode, ProductId, Sku, VariantId};

/// ProductVariant entity representing a specific variation of a product.
/// Variants can have their own SKU, barcode, and price overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariant {
    id: VariantId,
    product_id: ProductId,
    sku: Sku,
    barcode: Option<Barcode>,
    name: String,
    variant_attributes: JsonValue,
    price: Option<Decimal>,
    cost_price: Option<Decimal>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProductVariant {
    /// Creates a new ProductVariant with auto-generated SKU derived from parent
    pub fn create(
        product_id: ProductId,
        parent_sku: &Sku,
        variant_index: u32,
        name: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: VariantId::new(),
            product_id,
            sku: Sku::generate_variant(parent_sku, variant_index),
            barcode: None,
            name,
            variant_attributes: JsonValue::Object(Default::default()),
            price: None,
            cost_price: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a ProductVariant from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: VariantId,
        product_id: ProductId,
        sku: Sku,
        barcode: Option<Barcode>,
        name: String,
        variant_attributes: JsonValue,
        price: Option<Decimal>,
        cost_price: Option<Decimal>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            product_id,
            sku,
            barcode,
            name,
            variant_attributes,
            price,
            cost_price,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Returns the effective price (variant price if set, otherwise None - caller should use product base_price)
    pub fn effective_price(&self, product_base_price: Decimal) -> Decimal {
        self.price.unwrap_or(product_base_price)
    }

    /// Returns the effective cost (variant cost if set, otherwise None - caller should use product cost_price)
    pub fn effective_cost(&self, product_cost_price: Decimal) -> Decimal {
        self.cost_price.unwrap_or(product_cost_price)
    }

    /// Deactivates the variant without deleting it
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Activates the variant
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> VariantId {
        self.id
    }

    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    pub fn sku(&self) -> &Sku {
        &self.sku
    }

    pub fn barcode(&self) -> Option<&Barcode> {
        self.barcode.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variant_attributes(&self) -> &JsonValue {
        &self.variant_attributes
    }

    pub fn price(&self) -> Option<Decimal> {
        self.price
    }

    pub fn cost_price(&self) -> Option<Decimal> {
        self.cost_price
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_barcode(&mut self, barcode: Option<Barcode>) {
        self.barcode = barcode;
        self.updated_at = Utc::now();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn set_variant_attributes(&mut self, attributes: JsonValue) {
        self.variant_attributes = attributes;
        self.updated_at = Utc::now();
    }

    pub fn set_price(&mut self, price: Option<Decimal>) {
        self.price = price;
        self.updated_at = Utc::now();
    }

    pub fn set_cost_price(&mut self, cost_price: Option<Decimal>) {
        self.cost_price = cost_price;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_variant() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-ELE-ABC123-XY12".to_string());
        
        let variant = ProductVariant::create(
            product_id,
            &parent_sku,
            1,
            "Red - Large".to_string(),
        );
        
        assert_eq!(variant.product_id(), product_id);
        assert_eq!(variant.name(), "Red - Large");
        assert_eq!(variant.sku().as_str(), "PRD-ELE-ABC123-XY12-V001");
        assert!(variant.is_active());
        assert!(variant.price().is_none());
        assert!(variant.cost_price().is_none());
    }

    #[test]
    fn test_variant_sku_generation() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-GEN-TEST-1234".to_string());
        
        let variant1 = ProductVariant::create(product_id, &parent_sku, 1, "V1".to_string());
        let variant2 = ProductVariant::create(product_id, &parent_sku, 2, "V2".to_string());
        let variant10 = ProductVariant::create(product_id, &parent_sku, 10, "V10".to_string());
        
        assert_eq!(variant1.sku().as_str(), "PRD-GEN-TEST-1234-V001");
        assert_eq!(variant2.sku().as_str(), "PRD-GEN-TEST-1234-V002");
        assert_eq!(variant10.sku().as_str(), "PRD-GEN-TEST-1234-V010");
    }

    #[test]
    fn test_effective_price_with_override() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-TEST-123".to_string());
        let mut variant = ProductVariant::create(product_id, &parent_sku, 1, "Test".to_string());
        
        let product_base_price = dec!(100.00);
        
        // Without override, should use product price
        assert_eq!(variant.effective_price(product_base_price), dec!(100.00));
        
        // With override, should use variant price
        variant.set_price(Some(dec!(120.00)));
        assert_eq!(variant.effective_price(product_base_price), dec!(120.00));
    }

    #[test]
    fn test_effective_cost_with_override() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-TEST-123".to_string());
        let mut variant = ProductVariant::create(product_id, &parent_sku, 1, "Test".to_string());
        
        let product_cost_price = dec!(50.00);
        
        // Without override, should use product cost
        assert_eq!(variant.effective_cost(product_cost_price), dec!(50.00));
        
        // With override, should use variant cost
        variant.set_cost_price(Some(dec!(55.00)));
        assert_eq!(variant.effective_cost(product_cost_price), dec!(55.00));
    }

    #[test]
    fn test_deactivate_activate() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-TEST-123".to_string());
        let mut variant = ProductVariant::create(product_id, &parent_sku, 1, "Test".to_string());
        
        assert!(variant.is_active());
        
        variant.deactivate();
        assert!(!variant.is_active());
        
        variant.activate();
        assert!(variant.is_active());
    }

    #[test]
    fn test_set_variant_attributes() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-TEST-123".to_string());
        let mut variant = ProductVariant::create(product_id, &parent_sku, 1, "Test".to_string());
        
        let attrs = serde_json::json!({
            "color": "red",
            "size": "XL"
        });
        variant.set_variant_attributes(attrs.clone());
        assert_eq!(variant.variant_attributes(), &attrs);
    }

    #[test]
    fn test_set_barcode() {
        let product_id = ProductId::new();
        let parent_sku = Sku::from_string("PRD-TEST-123".to_string());
        let mut variant = ProductVariant::create(product_id, &parent_sku, 1, "Test".to_string());
        
        assert!(variant.barcode().is_none());
        
        let barcode = Barcode::new("9876543210123").unwrap();
        variant.set_barcode(Some(barcode));
        assert_eq!(variant.barcode().map(|b| b.as_str()), Some("9876543210123"));
    }
}
