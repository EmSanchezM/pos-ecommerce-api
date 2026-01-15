// RecipeIngredient entity - component required by a recipe

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{IngredientId, ProductId, RecipeId, UnitOfMeasure, VariantId};
use crate::InventoryError;

/// RecipeIngredient entity representing a component required by a recipe.
/// Stores quantity, unit of measure, and optional substitution settings.
///
/// Invariants:
/// - Either ingredient_product_id OR ingredient_variant_id must be set, but not both (XOR constraint)
/// - quantity must be positive
/// - estimated_waste_percentage must be between 0 and 1 (0% to 100%)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    id: IngredientId,
    recipe_id: RecipeId,
    ingredient_product_id: Option<ProductId>,
    ingredient_variant_id: Option<VariantId>,
    quantity: Decimal,
    unit_of_measure: UnitOfMeasure,
    is_optional: bool,
    can_substitute: bool,
    sort_order: i32,
    preparation_step: Option<String>,
    estimated_cost_per_unit: Option<Decimal>,
    estimated_waste_percentage: Decimal,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl RecipeIngredient {
    /// Creates a new RecipeIngredient for a product
    pub fn create_for_product(
        recipe_id: RecipeId,
        ingredient_product_id: ProductId,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
    ) -> Result<Self, InventoryError> {
        if quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidIngredientQuantity);
        }

        let now = Utc::now();
        Ok(Self {
            id: IngredientId::new(),
            recipe_id,
            ingredient_product_id: Some(ingredient_product_id),
            ingredient_variant_id: None,
            quantity,
            unit_of_measure,
            is_optional: false,
            can_substitute: false,
            sort_order: 0,
            preparation_step: None,
            estimated_cost_per_unit: None,
            estimated_waste_percentage: Decimal::ZERO,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }


    /// Creates a new RecipeIngredient for a variant
    pub fn create_for_variant(
        recipe_id: RecipeId,
        ingredient_variant_id: VariantId,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
    ) -> Result<Self, InventoryError> {
        if quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidIngredientQuantity);
        }

        let now = Utc::now();
        Ok(Self {
            id: IngredientId::new(),
            recipe_id,
            ingredient_product_id: None,
            ingredient_variant_id: Some(ingredient_variant_id),
            quantity,
            unit_of_measure,
            is_optional: false,
            can_substitute: false,
            sort_order: 0,
            preparation_step: None,
            estimated_cost_per_unit: None,
            estimated_waste_percentage: Decimal::ZERO,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a RecipeIngredient from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: IngredientId,
        recipe_id: RecipeId,
        ingredient_product_id: Option<ProductId>,
        ingredient_variant_id: Option<VariantId>,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        is_optional: bool,
        can_substitute: bool,
        sort_order: i32,
        preparation_step: Option<String>,
        estimated_cost_per_unit: Option<Decimal>,
        estimated_waste_percentage: Decimal,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        // Validate XOR constraint
        Self::validate_product_variant_constraint(ingredient_product_id, ingredient_variant_id)?;

        Ok(Self {
            id,
            recipe_id,
            ingredient_product_id,
            ingredient_variant_id,
            quantity,
            unit_of_measure,
            is_optional,
            can_substitute,
            sort_order,
            preparation_step,
            estimated_cost_per_unit,
            estimated_waste_percentage,
            notes,
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

    /// Calculates the effective cost for this ingredient including waste
    /// Formula: quantity * (1 + waste_percentage) * cost_per_unit
    pub fn calculate_effective_cost(&self) -> Option<Decimal> {
        self.estimated_cost_per_unit.map(|cost| {
            self.quantity * (Decimal::ONE + self.estimated_waste_percentage) * cost
        })
    }


    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> IngredientId {
        self.id
    }

    pub fn recipe_id(&self) -> RecipeId {
        self.recipe_id
    }

    pub fn ingredient_product_id(&self) -> Option<ProductId> {
        self.ingredient_product_id
    }

    pub fn ingredient_variant_id(&self) -> Option<VariantId> {
        self.ingredient_variant_id
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn unit_of_measure(&self) -> UnitOfMeasure {
        self.unit_of_measure
    }

    pub fn is_optional(&self) -> bool {
        self.is_optional
    }

    pub fn can_substitute(&self) -> bool {
        self.can_substitute
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }

    pub fn preparation_step(&self) -> Option<&str> {
        self.preparation_step.as_deref()
    }

    pub fn estimated_cost_per_unit(&self) -> Option<Decimal> {
        self.estimated_cost_per_unit
    }

    pub fn estimated_waste_percentage(&self) -> Decimal {
        self.estimated_waste_percentage
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
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

    pub fn set_quantity(&mut self, quantity: Decimal) -> Result<(), InventoryError> {
        if quantity <= Decimal::ZERO {
            return Err(InventoryError::InvalidIngredientQuantity);
        }
        self.quantity = quantity;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_unit_of_measure(&mut self, unit_of_measure: UnitOfMeasure) {
        self.unit_of_measure = unit_of_measure;
        self.updated_at = Utc::now();
    }

    pub fn set_optional(&mut self, is_optional: bool) {
        self.is_optional = is_optional;
        self.updated_at = Utc::now();
    }

    pub fn set_can_substitute(&mut self, can_substitute: bool) {
        self.can_substitute = can_substitute;
        self.updated_at = Utc::now();
    }

    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
    }

    pub fn set_preparation_step(&mut self, preparation_step: Option<String>) {
        self.preparation_step = preparation_step;
        self.updated_at = Utc::now();
    }

    pub fn set_estimated_cost_per_unit(&mut self, cost: Option<Decimal>) {
        self.estimated_cost_per_unit = cost;
        self.updated_at = Utc::now();
    }

    pub fn set_estimated_waste_percentage(&mut self, percentage: Decimal) -> Result<(), InventoryError> {
        if percentage < Decimal::ZERO || percentage > Decimal::ONE {
            return Err(InventoryError::InvalidWastePercentage);
        }
        self.estimated_waste_percentage = percentage;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_ingredient_for_product() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let ingredient = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(2.5),
            UnitOfMeasure::Kg,
        ).unwrap();

        assert_eq!(ingredient.recipe_id(), recipe_id);
        assert_eq!(ingredient.ingredient_product_id(), Some(product_id));
        assert!(ingredient.ingredient_variant_id().is_none());
        assert_eq!(ingredient.quantity(), dec!(2.5));
        assert_eq!(ingredient.unit_of_measure(), UnitOfMeasure::Kg);
        assert!(!ingredient.is_optional());
        assert!(!ingredient.can_substitute());
        assert_eq!(ingredient.sort_order(), 0);
        assert_eq!(ingredient.estimated_waste_percentage(), Decimal::ZERO);
    }

    #[test]
    fn test_create_ingredient_for_variant() {
        let recipe_id = RecipeId::new();
        let variant_id = VariantId::new();

        let ingredient = RecipeIngredient::create_for_variant(
            recipe_id,
            variant_id,
            dec!(1.0),
            UnitOfMeasure::Liter,
        ).unwrap();

        assert!(ingredient.ingredient_product_id().is_none());
        assert_eq!(ingredient.ingredient_variant_id(), Some(variant_id));
        assert_eq!(ingredient.quantity(), dec!(1.0));
        assert_eq!(ingredient.unit_of_measure(), UnitOfMeasure::Liter);
    }

    #[test]
    fn test_create_ingredient_invalid_quantity() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let result = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(0),
            UnitOfMeasure::Unit,
        );
        assert!(matches!(result, Err(InventoryError::InvalidIngredientQuantity)));

        let result = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(-1),
            UnitOfMeasure::Unit,
        );
        assert!(matches!(result, Err(InventoryError::InvalidIngredientQuantity)));
    }

    #[test]
    fn test_calculate_effective_cost() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let mut ingredient = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(2),
            UnitOfMeasure::Kg,
        ).unwrap();

        // No cost set
        assert!(ingredient.calculate_effective_cost().is_none());

        // Set cost without waste
        ingredient.set_estimated_cost_per_unit(Some(dec!(10)));
        // 2 * (1 + 0) * 10 = 20
        assert_eq!(ingredient.calculate_effective_cost(), Some(dec!(20)));

        // Set 10% waste
        ingredient.set_estimated_waste_percentage(dec!(0.1)).unwrap();
        // 2 * (1 + 0.1) * 10 = 22
        assert_eq!(ingredient.calculate_effective_cost(), Some(dec!(22)));
    }

    #[test]
    fn test_setters() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let mut ingredient = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(1),
            UnitOfMeasure::Unit,
        ).unwrap();

        ingredient.set_quantity(dec!(5)).unwrap();
        assert_eq!(ingredient.quantity(), dec!(5));

        ingredient.set_unit_of_measure(UnitOfMeasure::Lb);
        assert_eq!(ingredient.unit_of_measure(), UnitOfMeasure::Lb);

        ingredient.set_optional(true);
        assert!(ingredient.is_optional());

        ingredient.set_can_substitute(true);
        assert!(ingredient.can_substitute());

        ingredient.set_sort_order(3);
        assert_eq!(ingredient.sort_order(), 3);

        ingredient.set_preparation_step(Some("Dice finely".to_string()));
        assert_eq!(ingredient.preparation_step(), Some("Dice finely"));

        ingredient.set_estimated_cost_per_unit(Some(dec!(5.50)));
        assert_eq!(ingredient.estimated_cost_per_unit(), Some(dec!(5.50)));

        ingredient.set_estimated_waste_percentage(dec!(0.05)).unwrap();
        assert_eq!(ingredient.estimated_waste_percentage(), dec!(0.05));

        ingredient.set_notes(Some("Use fresh".to_string()));
        assert_eq!(ingredient.notes(), Some("Use fresh"));
    }

    #[test]
    fn test_set_quantity_invalid() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let mut ingredient = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(5),
            UnitOfMeasure::Unit,
        ).unwrap();

        let result = ingredient.set_quantity(dec!(0));
        assert!(matches!(result, Err(InventoryError::InvalidIngredientQuantity)));

        let result = ingredient.set_quantity(dec!(-1));
        assert!(matches!(result, Err(InventoryError::InvalidIngredientQuantity)));

        // Original value unchanged
        assert_eq!(ingredient.quantity(), dec!(5));
    }

    #[test]
    fn test_set_waste_percentage_invalid() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();

        let mut ingredient = RecipeIngredient::create_for_product(
            recipe_id,
            product_id,
            dec!(1),
            UnitOfMeasure::Unit,
        ).unwrap();

        let result = ingredient.set_estimated_waste_percentage(dec!(-0.1));
        assert!(matches!(result, Err(InventoryError::InvalidWastePercentage)));

        let result = ingredient.set_estimated_waste_percentage(dec!(1.5));
        assert!(matches!(result, Err(InventoryError::InvalidWastePercentage)));

        // Original value unchanged
        assert_eq!(ingredient.estimated_waste_percentage(), Decimal::ZERO);
    }

    #[test]
    fn test_xor_constraint_both_set() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();
        let variant_id = VariantId::new();
        let now = Utc::now();

        let result = RecipeIngredient::reconstitute(
            IngredientId::new(),
            recipe_id,
            Some(product_id),
            Some(variant_id),
            dec!(1),
            UnitOfMeasure::Unit,
            false,
            false,
            0,
            None,
            None,
            Decimal::ZERO,
            None,
            now,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_xor_constraint_neither_set() {
        let recipe_id = RecipeId::new();
        let now = Utc::now();

        let result = RecipeIngredient::reconstitute(
            IngredientId::new(),
            recipe_id,
            None,
            None,
            dec!(1),
            UnitOfMeasure::Unit,
            false,
            false,
            0,
            None,
            None,
            Decimal::ZERO,
            None,
            now,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_reconstitute_valid() {
        let recipe_id = RecipeId::new();
        let product_id = ProductId::new();
        let now = Utc::now();

        let ingredient = RecipeIngredient::reconstitute(
            IngredientId::new(),
            recipe_id,
            Some(product_id),
            None,
            dec!(3.5),
            UnitOfMeasure::Oz,
            true,
            true,
            2,
            Some("Chop coarsely".to_string()),
            Some(dec!(2.50)),
            dec!(0.15),
            Some("Organic preferred".to_string()),
            now,
            now,
        ).unwrap();

        assert_eq!(ingredient.recipe_id(), recipe_id);
        assert_eq!(ingredient.ingredient_product_id(), Some(product_id));
        assert!(ingredient.ingredient_variant_id().is_none());
        assert_eq!(ingredient.quantity(), dec!(3.5));
        assert_eq!(ingredient.unit_of_measure(), UnitOfMeasure::Oz);
        assert!(ingredient.is_optional());
        assert!(ingredient.can_substitute());
        assert_eq!(ingredient.sort_order(), 2);
        assert_eq!(ingredient.preparation_step(), Some("Chop coarsely"));
        assert_eq!(ingredient.estimated_cost_per_unit(), Some(dec!(2.50)));
        assert_eq!(ingredient.estimated_waste_percentage(), dec!(0.15));
        assert_eq!(ingredient.notes(), Some("Organic preferred"));
    }
}
