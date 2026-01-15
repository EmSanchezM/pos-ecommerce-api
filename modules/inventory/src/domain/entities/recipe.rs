// Recipe entity - Bill of Materials (BOM) for composite products

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{ProductId, RecipeId, VariantId};
use crate::InventoryError;

/// Recipe entity representing a Bill of Materials (BOM) for composite products.
/// Defines ingredients needed to produce a product or variant.
///
/// Invariants:
/// - Either product_id OR variant_id must be set, but not both (XOR constraint)
/// - Only one active recipe per product/variant at any time
/// - yield_quantity must be positive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    id: RecipeId,
    product_id: Option<ProductId>,
    variant_id: Option<VariantId>,
    name: String,
    description: Option<String>,
    version: i32,
    yield_quantity: Decimal,
    is_active: bool,
    preparation_time_minutes: Option<i32>,
    calculate_cost_from_ingredients: bool,
    notes: Option<String>,
    metadata: JsonValue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Recipe {
    /// Creates a new Recipe for a product
    pub fn create_for_product(
        product_id: ProductId,
        name: String,
        yield_quantity: Decimal,
    ) -> Result<Self, InventoryError> {
        if yield_quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidYieldQuantity);
        }

        let now = Utc::now();
        Ok(Self {
            id: RecipeId::new(),
            product_id: Some(product_id),
            variant_id: None,
            name,
            description: None,
            version: 1,
            yield_quantity,
            is_active: true,
            preparation_time_minutes: None,
            calculate_cost_from_ingredients: true,
            notes: None,
            metadata: JsonValue::Object(Default::default()),
            created_at: now,
            updated_at: now,
        })
    }


    /// Creates a new Recipe for a variant
    pub fn create_for_variant(
        variant_id: VariantId,
        name: String,
        yield_quantity: Decimal,
    ) -> Result<Self, InventoryError> {
        if yield_quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidYieldQuantity);
        }

        let now = Utc::now();
        Ok(Self {
            id: RecipeId::new(),
            product_id: None,
            variant_id: Some(variant_id),
            name,
            description: None,
            version: 1,
            yield_quantity,
            is_active: true,
            preparation_time_minutes: None,
            calculate_cost_from_ingredients: true,
            notes: None,
            metadata: JsonValue::Object(Default::default()),
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a Recipe from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: RecipeId,
        product_id: Option<ProductId>,
        variant_id: Option<VariantId>,
        name: String,
        description: Option<String>,
        version: i32,
        yield_quantity: Decimal,
        is_active: bool,
        preparation_time_minutes: Option<i32>,
        calculate_cost_from_ingredients: bool,
        notes: Option<String>,
        metadata: JsonValue,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        // Validate XOR constraint
        Self::validate_product_variant_constraint(product_id, variant_id)?;

        Ok(Self {
            id,
            product_id,
            variant_id,
            name,
            description,
            version,
            yield_quantity,
            is_active,
            preparation_time_minutes,
            calculate_cost_from_ingredients,
            notes,
            metadata,
            created_at,
            updated_at,
        })
    }

    /// Validates that exactly one of product_id or variant_id is set
    fn validate_product_variant_constraint(
        product_id: Option<ProductId>,
        variant_id: Option<VariantId>,
    ) -> Result<(), InventoryError> {
        match (product_id, variant_id) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            _ => Err(InventoryError::InvalidProductVariantConstraint),
        }
    }

    /// Deactivates the recipe without deleting it
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Activates the recipe
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Calculates the cost per unit based on ingredient costs
    /// Returns the total ingredient cost divided by yield_quantity
    /// 
    /// Formula: sum(ingredient_quantity * (1 + waste_percentage) * ingredient_cost) / yield_quantity
    pub fn calculate_cost(&self, total_ingredient_cost: Decimal) -> Decimal {
        if self.yield_quantity <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        total_ingredient_cost / self.yield_quantity
    }

    /// Increments the version number for recipe versioning
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }


    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> RecipeId {
        self.id
    }

    pub fn product_id(&self) -> Option<ProductId> {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<VariantId> {
        self.variant_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn yield_quantity(&self) -> Decimal {
        self.yield_quantity
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn preparation_time_minutes(&self) -> Option<i32> {
        self.preparation_time_minutes
    }

    pub fn calculate_cost_from_ingredients(&self) -> bool {
        self.calculate_cost_from_ingredients
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn metadata(&self) -> &JsonValue {
        &self.metadata
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

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_yield_quantity(&mut self, yield_quantity: Decimal) -> Result<(), InventoryError> {
        if yield_quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidYieldQuantity);
        }
        self.yield_quantity = yield_quantity;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_preparation_time_minutes(&mut self, minutes: Option<i32>) {
        self.preparation_time_minutes = minutes;
        self.updated_at = Utc::now();
    }

    pub fn set_calculate_cost_from_ingredients(&mut self, calculate: bool) {
        self.calculate_cost_from_ingredients = calculate;
        self.updated_at = Utc::now();
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    pub fn set_metadata(&mut self, metadata: JsonValue) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_recipe_for_product() {
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(
            product_id,
            "Chocolate Cake".to_string(),
            dec!(10),
        ).unwrap();

        assert_eq!(recipe.product_id(), Some(product_id));
        assert!(recipe.variant_id().is_none());
        assert_eq!(recipe.name(), "Chocolate Cake");
        assert_eq!(recipe.yield_quantity(), dec!(10));
        assert!(recipe.is_active());
        assert_eq!(recipe.version(), 1);
        assert!(recipe.calculate_cost_from_ingredients());
    }

    #[test]
    fn test_create_recipe_for_variant() {
        let variant_id = VariantId::new();
        let recipe = Recipe::create_for_variant(
            variant_id,
            "Large Chocolate Cake".to_string(),
            dec!(15),
        ).unwrap();

        assert!(recipe.product_id().is_none());
        assert_eq!(recipe.variant_id(), Some(variant_id));
        assert_eq!(recipe.name(), "Large Chocolate Cake");
        assert_eq!(recipe.yield_quantity(), dec!(15));
    }

    #[test]
    fn test_create_recipe_invalid_yield_quantity() {
        let product_id = ProductId::new();
        
        let result = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(0),
        );
        assert!(matches!(result, Err(InventoryError::InvalidYieldQuantity)));

        let result = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(-5),
        );
        assert!(matches!(result, Err(InventoryError::InvalidYieldQuantity)));
    }

    #[test]
    fn test_deactivate_activate() {
        let product_id = ProductId::new();
        let mut recipe = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(1),
        ).unwrap();

        assert!(recipe.is_active());

        recipe.deactivate();
        assert!(!recipe.is_active());

        recipe.activate();
        assert!(recipe.is_active());
    }

    #[test]
    fn test_calculate_cost() {
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(10),
        ).unwrap();

        // Total ingredient cost of 100, yield of 10 = cost per unit of 10
        let cost_per_unit = recipe.calculate_cost(dec!(100));
        assert_eq!(cost_per_unit, dec!(10));

        // Total ingredient cost of 50, yield of 10 = cost per unit of 5
        let cost_per_unit = recipe.calculate_cost(dec!(50));
        assert_eq!(cost_per_unit, dec!(5));
    }

    #[test]
    fn test_increment_version() {
        let product_id = ProductId::new();
        let mut recipe = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(1),
        ).unwrap();

        assert_eq!(recipe.version(), 1);

        recipe.increment_version();
        assert_eq!(recipe.version(), 2);

        recipe.increment_version();
        assert_eq!(recipe.version(), 3);
    }

    #[test]
    fn test_setters() {
        let product_id = ProductId::new();
        let mut recipe = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(1),
        ).unwrap();

        recipe.set_name("Updated Recipe".to_string());
        assert_eq!(recipe.name(), "Updated Recipe");

        recipe.set_description(Some("A delicious recipe".to_string()));
        assert_eq!(recipe.description(), Some("A delicious recipe"));

        recipe.set_yield_quantity(dec!(20)).unwrap();
        assert_eq!(recipe.yield_quantity(), dec!(20));

        recipe.set_preparation_time_minutes(Some(45));
        assert_eq!(recipe.preparation_time_minutes(), Some(45));

        recipe.set_calculate_cost_from_ingredients(false);
        assert!(!recipe.calculate_cost_from_ingredients());

        recipe.set_notes(Some("Special instructions".to_string()));
        assert_eq!(recipe.notes(), Some("Special instructions"));

        let metadata = serde_json::json!({"difficulty": "medium"});
        recipe.set_metadata(metadata.clone());
        assert_eq!(recipe.metadata(), &metadata);
    }

    #[test]
    fn test_set_yield_quantity_invalid() {
        let product_id = ProductId::new();
        let mut recipe = Recipe::create_for_product(
            product_id,
            "Test Recipe".to_string(),
            dec!(10),
        ).unwrap();

        let result = recipe.set_yield_quantity(dec!(0));
        assert!(matches!(result, Err(InventoryError::InvalidYieldQuantity)));

        let result = recipe.set_yield_quantity(dec!(-5));
        assert!(matches!(result, Err(InventoryError::InvalidYieldQuantity)));

        // Original value should be unchanged
        assert_eq!(recipe.yield_quantity(), dec!(10));
    }

    #[test]
    fn test_xor_constraint_both_set() {
        let product_id = ProductId::new();
        let variant_id = VariantId::new();
        let now = Utc::now();

        let result = Recipe::reconstitute(
            RecipeId::new(),
            Some(product_id),
            Some(variant_id),
            "Test".to_string(),
            None,
            1,
            dec!(1),
            true,
            None,
            true,
            None,
            JsonValue::Object(Default::default()),
            now,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_xor_constraint_neither_set() {
        let now = Utc::now();

        let result = Recipe::reconstitute(
            RecipeId::new(),
            None,
            None,
            "Test".to_string(),
            None,
            1,
            dec!(1),
            true,
            None,
            true,
            None,
            JsonValue::Object(Default::default()),
            now,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_reconstitute_valid() {
        let product_id = ProductId::new();
        let now = Utc::now();

        let recipe = Recipe::reconstitute(
            RecipeId::new(),
            Some(product_id),
            None,
            "Reconstituted Recipe".to_string(),
            Some("Description".to_string()),
            3,
            dec!(5),
            true,
            Some(30),
            true,
            Some("Notes".to_string()),
            serde_json::json!({"key": "value"}),
            now,
            now,
        ).unwrap();

        assert_eq!(recipe.product_id(), Some(product_id));
        assert!(recipe.variant_id().is_none());
        assert_eq!(recipe.name(), "Reconstituted Recipe");
        assert_eq!(recipe.description(), Some("Description"));
        assert_eq!(recipe.version(), 3);
        assert_eq!(recipe.yield_quantity(), dec!(5));
        assert!(recipe.is_active());
        assert_eq!(recipe.preparation_time_minutes(), Some(30));
        assert!(recipe.calculate_cost_from_ingredients());
        assert_eq!(recipe.notes(), Some("Notes"));
    }
}
