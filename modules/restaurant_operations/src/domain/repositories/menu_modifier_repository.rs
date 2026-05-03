use async_trait::async_trait;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::{MenuModifier, MenuModifierGroup};
use crate::domain::value_objects::{MenuModifierGroupId, MenuModifierId};

/// Bundle returned by `list_groups_with_modifiers` so the API can render the
/// nested structure without N+1.
#[derive(Debug, Clone)]
pub struct ModifierGroupWithModifiers {
    pub group: MenuModifierGroup,
    pub modifiers: Vec<MenuModifier>,
}

#[async_trait]
pub trait MenuModifierRepository: Send + Sync {
    // Groups
    async fn save_group(&self, group: &MenuModifierGroup) -> Result<(), RestaurantOperationsError>;
    async fn update_group(
        &self,
        group: &MenuModifierGroup,
    ) -> Result<(), RestaurantOperationsError>;
    async fn find_group(
        &self,
        id: MenuModifierGroupId,
    ) -> Result<Option<MenuModifierGroup>, RestaurantOperationsError>;
    async fn list_groups_with_modifiers(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError>;

    // Modifiers
    async fn save_modifier(&self, modifier: &MenuModifier)
    -> Result<(), RestaurantOperationsError>;
    async fn update_modifier(
        &self,
        modifier: &MenuModifier,
    ) -> Result<(), RestaurantOperationsError>;
    async fn find_modifier(
        &self,
        id: MenuModifierId,
    ) -> Result<Option<MenuModifier>, RestaurantOperationsError>;
    async fn list_modifiers_by_group(
        &self,
        group_id: MenuModifierGroupId,
    ) -> Result<Vec<MenuModifier>, RestaurantOperationsError>;
    /// Resolve `ids` against `menu_modifiers` and return them in the requested
    /// order. Used by the create-ticket use case to build the
    /// `modifiers_summary` text for a KDS item.
    async fn find_modifiers_in(
        &self,
        ids: &[MenuModifierId],
    ) -> Result<Vec<MenuModifier>, RestaurantOperationsError>;

    // Product M2M
    async fn assign_groups_to_product(
        &self,
        product_id: Uuid,
        group_ids: &[MenuModifierGroupId],
    ) -> Result<(), RestaurantOperationsError>;
    async fn list_groups_for_product(
        &self,
        product_id: Uuid,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError>;
}
