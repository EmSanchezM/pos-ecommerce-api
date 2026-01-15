// IngredientSubstitute entity - alternative ingredient for a recipe ingredient

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{IngredientId, ProductId, SubstituteId, VariantId};
use crate::InventoryError;

/// IngredientSubstitute entity representing an alternative ingredient.
/// Allows defining substitute products/variants with conversion ratios.
///
/// Invariants:
/// - Either substitute_product_id OR substitute_variant_id must be set, but not both (XOR constraint)
/// - conversion_ratio must be positive
/// - priority must be non-negative (lower number = higher preference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientSubstitute {
    id: SubstituteId,
    recipe_ingredient_id: IngredientId,
    substitute_product_id: Option<ProductId>,
    substitute_variant_id: Option<VariantId>,
    conversion_ratio: Decimal,
    priority: i32,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl IngredientSubstitute {
    /// Creates a new IngredientSubstitute for a product
    pub fn create_for_product(
        recipe_ingredient_id: IngredientId,
        substitute_product_id: ProductId,
        conversion_ratio: Decimal,
        priority: i32,
    ) -> Result<Self, InventoryError> {
        if conversion_ratio <= Decimal::ZERO {
            return Err(InventoryError::InvalidConversionRatio);
        }
        if priority < 0 {
            return Err(InventoryError::InvalidSubstitutePriority);
        }

        Ok(Self {
            id: SubstituteId::new(),
            recipe_ingredient_id,
            substitute_product_id: Some(substitute_product_id),
            substitute_variant_id: None,
            conversion_ratio,
            priority,
            notes: None,
            created_at: Utc::now(),
        })
    }


    /// Creates a new IngredientSubstitute for a variant
    pub fn create_for_variant(
        recipe_ingredient_id: IngredientId,
        substitute_variant_id: VariantId,
        conversion_ratio: Decimal,
        priority: i32,
    ) -> Result<Self, InventoryError> {
        if conversion_ratio <= Decimal::ZERO {
            return Err(InventoryError::InvalidConversionRatio);
        }
        if priority < 0 {
            return Err(InventoryError::InvalidSubstitutePriority);
        }

        Ok(Self {
            id: SubstituteId::new(),
            recipe_ingredient_id,
            substitute_product_id: None,
            substitute_variant_id: Some(substitute_variant_id),
            conversion_ratio,
            priority,
            notes: None,
            created_at: Utc::now(),
        })
    }

    /// Reconstitutes an IngredientSubstitute from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SubstituteId,
        recipe_ingredient_id: IngredientId,
        substitute_product_id: Option<ProductId>,
        substitute_variant_id: Option<VariantId>,
        conversion_ratio: Decimal,
        priority: i32,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        // Validate XOR constraint
        Self::validate_product_variant_constraint(substitute_product_id, substitute_variant_id)?;

        Ok(Self {
            id,
            recipe_ingredient_id,
            substitute_product_id,
            substitute_variant_id,
            conversion_ratio,
            priority,
            notes,
            created_at,
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

    /// Calculates the substitute quantity based on original quantity and conversion ratio
    /// substitute_quantity = original_quantity * conversion_ratio
    pub fn calculate_substitute_quantity(&self, original_quantity: Decimal) -> Decimal {
        original_quantity * self.conversion_ratio
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> SubstituteId {
        self.id
    }

    pub fn recipe_ingredient_id(&self) -> IngredientId {
        self.recipe_ingredient_id
    }

    pub fn substitute_product_id(&self) -> Option<ProductId> {
        self.substitute_product_id
    }

    pub fn substitute_variant_id(&self) -> Option<VariantId> {
        self.substitute_variant_id
    }

    pub fn conversion_ratio(&self) -> Decimal {
        self.conversion_ratio
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_conversion_ratio(&mut self, ratio: Decimal) -> Result<(), InventoryError> {
        if ratio <= Decimal::ZERO {
            return Err(InventoryError::InvalidConversionRatio);
        }
        self.conversion_ratio = ratio;
        Ok(())
    }

    pub fn set_priority(&mut self, priority: i32) -> Result<(), InventoryError> {
        if priority < 0 {
            return Err(InventoryError::InvalidSubstitutePriority);
        }
        self.priority = priority;
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_substitute_for_product() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1.5),
            1,
        ).unwrap();

        assert_eq!(substitute.recipe_ingredient_id(), ingredient_id);
        assert_eq!(substitute.substitute_product_id(), Some(product_id));
        assert!(substitute.substitute_variant_id().is_none());
        assert_eq!(substitute.conversion_ratio(), dec!(1.5));
        assert_eq!(substitute.priority(), 1);
        assert!(substitute.notes().is_none());
    }

    #[test]
    fn test_create_substitute_for_variant() {
        let ingredient_id = IngredientId::new();
        let variant_id = VariantId::new();

        let substitute = IngredientSubstitute::create_for_variant(
            ingredient_id,
            variant_id,
            dec!(0.8),
            2,
        ).unwrap();

        assert!(substitute.substitute_product_id().is_none());
        assert_eq!(substitute.substitute_variant_id(), Some(variant_id));
        assert_eq!(substitute.conversion_ratio(), dec!(0.8));
        assert_eq!(substitute.priority(), 2);
    }

    #[test]
    fn test_create_substitute_invalid_conversion_ratio() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let result = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(0),
            1,
        );
        assert!(matches!(result, Err(InventoryError::InvalidConversionRatio)));

        let result = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(-1),
            1,
        );
        assert!(matches!(result, Err(InventoryError::InvalidConversionRatio)));
    }

    #[test]
    fn test_create_substitute_invalid_priority() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let result = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1),
            -1,
        );
        assert!(matches!(result, Err(InventoryError::InvalidSubstitutePriority)));
    }

    #[test]
    fn test_calculate_substitute_quantity() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1.5),
            1,
        ).unwrap();

        // 10 units * 1.5 ratio = 15 units
        assert_eq!(substitute.calculate_substitute_quantity(dec!(10)), dec!(15));

        // 4 units * 1.5 ratio = 6 units
        assert_eq!(substitute.calculate_substitute_quantity(dec!(4)), dec!(6));
    }

    #[test]
    fn test_setters() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let mut substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1),
            1,
        ).unwrap();

        substitute.set_conversion_ratio(dec!(2.0)).unwrap();
        assert_eq!(substitute.conversion_ratio(), dec!(2.0));

        substitute.set_priority(5).unwrap();
        assert_eq!(substitute.priority(), 5);

        substitute.set_notes(Some("Use when primary unavailable".to_string()));
        assert_eq!(substitute.notes(), Some("Use when primary unavailable"));
    }

    #[test]
    fn test_set_conversion_ratio_invalid() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let mut substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1),
            1,
        ).unwrap();

        let result = substitute.set_conversion_ratio(dec!(0));
        assert!(matches!(result, Err(InventoryError::InvalidConversionRatio)));

        let result = substitute.set_conversion_ratio(dec!(-0.5));
        assert!(matches!(result, Err(InventoryError::InvalidConversionRatio)));

        // Original value unchanged
        assert_eq!(substitute.conversion_ratio(), dec!(1));
    }

    #[test]
    fn test_set_priority_invalid() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        let mut substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1),
            1,
        ).unwrap();

        let result = substitute.set_priority(-1);
        assert!(matches!(result, Err(InventoryError::InvalidSubstitutePriority)));

        // Original value unchanged
        assert_eq!(substitute.priority(), 1);
    }

    #[test]
    fn test_xor_constraint_both_set() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();
        let variant_id = VariantId::new();
        let now = Utc::now();

        let result = IngredientSubstitute::reconstitute(
            SubstituteId::new(),
            ingredient_id,
            Some(product_id),
            Some(variant_id),
            dec!(1),
            1,
            None,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_xor_constraint_neither_set() {
        let ingredient_id = IngredientId::new();
        let now = Utc::now();

        let result = IngredientSubstitute::reconstitute(
            SubstituteId::new(),
            ingredient_id,
            None,
            None,
            dec!(1),
            1,
            None,
            now,
        );

        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_reconstitute_valid() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();
        let now = Utc::now();

        let substitute = IngredientSubstitute::reconstitute(
            SubstituteId::new(),
            ingredient_id,
            Some(product_id),
            None,
            dec!(1.25),
            3,
            Some("Premium alternative".to_string()),
            now,
        ).unwrap();

        assert_eq!(substitute.recipe_ingredient_id(), ingredient_id);
        assert_eq!(substitute.substitute_product_id(), Some(product_id));
        assert!(substitute.substitute_variant_id().is_none());
        assert_eq!(substitute.conversion_ratio(), dec!(1.25));
        assert_eq!(substitute.priority(), 3);
        assert_eq!(substitute.notes(), Some("Premium alternative"));
    }

    #[test]
    fn test_priority_zero_is_valid() {
        let ingredient_id = IngredientId::new();
        let product_id = ProductId::new();

        // Priority 0 should be valid (highest priority)
        let substitute = IngredientSubstitute::create_for_product(
            ingredient_id,
            product_id,
            dec!(1),
            0,
        ).unwrap();

        assert_eq!(substitute.priority(), 0);
    }
}
