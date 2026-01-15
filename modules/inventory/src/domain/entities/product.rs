// Product entity - main product catalog item

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{Barcode, CategoryId, Currency, ProductId, Sku, UnitOfMeasure};

/// Product entity representing a catalog item that can be sold.
/// Supports optional variants, inventory tracking, and flexible attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    id: ProductId,
    sku: Sku,
    barcode: Option<Barcode>,
    name: String,
    description: Option<String>,
    category_id: Option<CategoryId>,
    brand: Option<String>,
    unit_of_measure: UnitOfMeasure,
    base_price: Decimal,
    cost_price: Decimal,
    currency: Currency,
    is_perishable: bool,
    is_trackable: bool,
    has_variants: bool,
    tax_rate: Decimal,
    tax_included: bool,
    attributes: JsonValue,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Product {
    /// Creates a new Product with auto-generated SKU
    pub fn create(
        name: String,
        unit_of_measure: UnitOfMeasure,
        category_code: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ProductId::new(),
            sku: Sku::generate(category_code),
            barcode: None,
            name,
            description: None,
            category_id: None,
            brand: None,
            unit_of_measure,
            base_price: Decimal::ZERO,
            cost_price: Decimal::ZERO,
            currency: Currency::default(),
            is_perishable: false,
            is_trackable: true,
            has_variants: false,
            tax_rate: Decimal::ZERO,
            tax_included: false,
            attributes: JsonValue::Object(Default::default()),
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Product from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ProductId,
        sku: Sku,
        barcode: Option<Barcode>,
        name: String,
        description: Option<String>,
        category_id: Option<CategoryId>,
        brand: Option<String>,
        unit_of_measure: UnitOfMeasure,
        base_price: Decimal,
        cost_price: Decimal,
        currency: Currency,
        is_perishable: bool,
        is_trackable: bool,
        has_variants: bool,
        tax_rate: Decimal,
        tax_included: bool,
        attributes: JsonValue,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sku,
            barcode,
            name,
            description,
            category_id,
            brand,
            unit_of_measure,
            base_price,
            cost_price,
            currency,
            is_perishable,
            is_trackable,
            has_variants,
            tax_rate,
            tax_included,
            attributes,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Deactivates the product without deleting it
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Activates the product
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> ProductId {
        self.id
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

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn category_id(&self) -> Option<CategoryId> {
        self.category_id
    }

    pub fn brand(&self) -> Option<&str> {
        self.brand.as_deref()
    }

    pub fn unit_of_measure(&self) -> UnitOfMeasure {
        self.unit_of_measure
    }

    pub fn base_price(&self) -> Decimal {
        self.base_price
    }

    pub fn cost_price(&self) -> Decimal {
        self.cost_price
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn is_perishable(&self) -> bool {
        self.is_perishable
    }

    pub fn is_trackable(&self) -> bool {
        self.is_trackable
    }

    pub fn has_variants(&self) -> bool {
        self.has_variants
    }

    pub fn tax_rate(&self) -> Decimal {
        self.tax_rate
    }

    pub fn tax_included(&self) -> bool {
        self.tax_included
    }

    pub fn attributes(&self) -> &JsonValue {
        &self.attributes
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

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_category_id(&mut self, category_id: Option<CategoryId>) {
        self.category_id = category_id;
        self.updated_at = Utc::now();
    }

    pub fn set_brand(&mut self, brand: Option<String>) {
        self.brand = brand;
        self.updated_at = Utc::now();
    }

    pub fn set_unit_of_measure(&mut self, unit_of_measure: UnitOfMeasure) {
        self.unit_of_measure = unit_of_measure;
        self.updated_at = Utc::now();
    }

    pub fn set_base_price(&mut self, base_price: Decimal) {
        self.base_price = base_price;
        self.updated_at = Utc::now();
    }

    pub fn set_cost_price(&mut self, cost_price: Decimal) {
        self.cost_price = cost_price;
        self.updated_at = Utc::now();
    }

    pub fn set_currency(&mut self, currency: Currency) {
        self.currency = currency;
        self.updated_at = Utc::now();
    }

    pub fn set_perishable(&mut self, is_perishable: bool) {
        self.is_perishable = is_perishable;
        self.updated_at = Utc::now();
    }

    pub fn set_trackable(&mut self, is_trackable: bool) {
        self.is_trackable = is_trackable;
        self.updated_at = Utc::now();
    }

    pub fn set_has_variants(&mut self, has_variants: bool) {
        self.has_variants = has_variants;
        self.updated_at = Utc::now();
    }

    pub fn set_tax_rate(&mut self, tax_rate: Decimal) {
        self.tax_rate = tax_rate;
        self.updated_at = Utc::now();
    }

    pub fn set_tax_included(&mut self, tax_included: bool) {
        self.tax_included = tax_included;
        self.updated_at = Utc::now();
    }

    pub fn set_attributes(&mut self, attributes: JsonValue) {
        self.attributes = attributes;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_product() {
        let product = Product::create(
            "Test Product".to_string(),
            UnitOfMeasure::Unit,
            Some("Electronics"),
        );
        
        assert_eq!(product.name(), "Test Product");
        assert_eq!(product.unit_of_measure(), UnitOfMeasure::Unit);
        assert!(product.sku().as_str().starts_with("PRD-ELE-"));
        assert!(product.is_active());
        assert!(product.is_trackable());
        assert!(!product.is_perishable());
        assert!(!product.has_variants());
        assert_eq!(product.base_price(), Decimal::ZERO);
        assert_eq!(product.cost_price(), Decimal::ZERO);
    }

    #[test]
    fn test_create_product_without_category() {
        let product = Product::create(
            "Generic Product".to_string(),
            UnitOfMeasure::Kg,
            None,
        );
        
        assert!(product.sku().as_str().starts_with("PRD-GEN-"));
    }

    #[test]
    fn test_deactivate_activate() {
        let mut product = Product::create("Test".to_string(), UnitOfMeasure::Unit, None);
        assert!(product.is_active());
        
        product.deactivate();
        assert!(!product.is_active());
        
        product.activate();
        assert!(product.is_active());
    }

    #[test]
    fn test_setters() {
        let mut product = Product::create("Test".to_string(), UnitOfMeasure::Unit, None);
        
        product.set_name("Updated Name".to_string());
        assert_eq!(product.name(), "Updated Name");
        
        product.set_description(Some("A description".to_string()));
        assert_eq!(product.description(), Some("A description"));
        
        product.set_brand(Some("BrandX".to_string()));
        assert_eq!(product.brand(), Some("BrandX"));
        
        product.set_base_price(dec!(99.99));
        assert_eq!(product.base_price(), dec!(99.99));
        
        product.set_cost_price(dec!(50.00));
        assert_eq!(product.cost_price(), dec!(50.00));
        
        product.set_tax_rate(dec!(0.15));
        assert_eq!(product.tax_rate(), dec!(0.15));
        
        product.set_tax_included(true);
        assert!(product.tax_included());
        
        product.set_perishable(true);
        assert!(product.is_perishable());
        
        product.set_trackable(false);
        assert!(!product.is_trackable());
        
        product.set_has_variants(true);
        assert!(product.has_variants());
    }

    #[test]
    fn test_set_category() {
        let mut product = Product::create("Test".to_string(), UnitOfMeasure::Unit, None);
        assert!(product.category_id().is_none());
        
        let category_id = CategoryId::new();
        product.set_category_id(Some(category_id));
        assert_eq!(product.category_id(), Some(category_id));
    }

    #[test]
    fn test_set_barcode() {
        let mut product = Product::create("Test".to_string(), UnitOfMeasure::Unit, None);
        assert!(product.barcode().is_none());
        
        let barcode = Barcode::new("1234567890123").unwrap();
        product.set_barcode(Some(barcode.clone()));
        assert_eq!(product.barcode().map(|b| b.as_str()), Some("1234567890123"));
    }

    #[test]
    fn test_set_attributes() {
        let mut product = Product::create("Test".to_string(), UnitOfMeasure::Unit, None);
        
        let attrs = serde_json::json!({
            "color": "red",
            "size": "large"
        });
        product.set_attributes(attrs.clone());
        assert_eq!(product.attributes(), &attrs);
    }
}
