use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::RestaurantOperationsError;
use crate::domain::value_objects::{MenuModifierGroupId, MenuModifierId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuModifier {
    id: MenuModifierId,
    group_id: MenuModifierGroupId,
    name: String,
    price_delta: Decimal,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MenuModifier {
    pub fn new(
        group_id: MenuModifierGroupId,
        name: String,
        price_delta: Decimal,
        sort_order: i32,
    ) -> Result<Self, RestaurantOperationsError> {
        if name.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "name is required".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: MenuModifierId::new(),
            group_id,
            name,
            price_delta,
            sort_order,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: MenuModifierId,
        group_id: MenuModifierGroupId,
        name: String,
        price_delta: Decimal,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            group_id,
            name,
            price_delta,
            sort_order,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn update(
        &mut self,
        name: String,
        price_delta: Decimal,
        sort_order: i32,
        is_active: bool,
    ) -> Result<(), RestaurantOperationsError> {
        if name.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "name is required".to_string(),
            ));
        }
        self.name = name;
        self.price_delta = price_delta;
        self.sort_order = sort_order;
        self.is_active = is_active;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn id(&self) -> MenuModifierId {
        self.id
    }
    pub fn group_id(&self) -> MenuModifierGroupId {
        self.group_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn price_delta(&self) -> Decimal {
        self.price_delta
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
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
}
