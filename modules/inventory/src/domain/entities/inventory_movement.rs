// InventoryMovement entity - Kardex entry for stock changes

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::value_objects::{Currency, MovementId, MovementType, StockId};
use identity::UserId;

/// InventoryMovement entity representing a Kardex entry for any stock change.
/// Records all inventory movements for audit and cost tracking purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMovement {
    id: MovementId,
    stock_id: StockId,
    movement_type: MovementType,
    movement_reason: Option<String>,
    quantity: Decimal,
    unit_cost: Option<Decimal>,
    currency: Currency,
    balance_after: Decimal,
    reference_type: Option<String>,
    reference_id: Option<Uuid>,
    actor_id: UserId,
    notes: Option<String>,
    metadata: JsonValue,
    created_at: DateTime<Utc>,
}

impl InventoryMovement {
    /// Creates a new InventoryMovement (Kardex entry)
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        stock_id: StockId,
        movement_type: MovementType,
        movement_reason: Option<String>,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
        currency: Currency,
        balance_after: Decimal,
        reference_type: Option<String>,
        reference_id: Option<Uuid>,
        actor_id: UserId,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: MovementId::new(),
            stock_id,
            movement_type,
            movement_reason,
            quantity,
            unit_cost,
            currency,
            balance_after,
            reference_type,
            reference_id,
            actor_id,
            notes,
            metadata: JsonValue::Object(Default::default()),
            created_at: Utc::now(),
        }
    }

    /// Reconstitutes an InventoryMovement from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: MovementId,
        stock_id: StockId,
        movement_type: MovementType,
        movement_reason: Option<String>,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
        currency: Currency,
        balance_after: Decimal,
        reference_type: Option<String>,
        reference_id: Option<Uuid>,
        actor_id: UserId,
        notes: Option<String>,
        metadata: JsonValue,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            stock_id,
            movement_type,
            movement_reason,
            quantity,
            unit_cost,
            currency,
            balance_after,
            reference_type,
            reference_id,
            actor_id,
            notes,
            metadata,
            created_at,
        }
    }

    /// Computes the total cost (quantity * unit_cost)
    /// Returns None if unit_cost is not set
    pub fn total_cost(&self) -> Option<Decimal> {
        self.unit_cost.map(|cost| self.quantity.abs() * cost)
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> MovementId {
        self.id
    }

    pub fn stock_id(&self) -> StockId {
        self.stock_id
    }

    pub fn movement_type(&self) -> MovementType {
        self.movement_type
    }

    pub fn movement_reason(&self) -> Option<&str> {
        self.movement_reason.as_deref()
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn unit_cost(&self) -> Option<Decimal> {
        self.unit_cost
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn balance_after(&self) -> Decimal {
        self.balance_after
    }

    pub fn reference_type(&self) -> Option<&str> {
        self.reference_type.as_deref()
    }

    pub fn reference_id(&self) -> Option<Uuid> {
        self.reference_id
    }

    pub fn actor_id(&self) -> UserId {
        self.actor_id
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[test]
    fn test_create_movement() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let movement = InventoryMovement::create(
            stock_id,
            MovementType::In,
            Some("purchase".to_string()),
            dec!(100),
            Some(dec!(10.50)),
            Currency::usd(),
            dec!(100),
            Some("purchase_order".to_string()),
            Some(new_uuid()),
            actor_id,
            Some("Initial stock".to_string()),
        );
        
        assert_eq!(movement.stock_id(), stock_id);
        assert_eq!(movement.movement_type(), MovementType::In);
        assert_eq!(movement.movement_reason(), Some("purchase"));
        assert_eq!(movement.quantity(), dec!(100));
        assert_eq!(movement.unit_cost(), Some(dec!(10.50)));
        assert_eq!(movement.balance_after(), dec!(100));
        assert_eq!(movement.actor_id(), actor_id);
        assert_eq!(movement.notes(), Some("Initial stock"));
    }

    #[test]
    fn test_total_cost_calculation() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let movement = InventoryMovement::create(
            stock_id,
            MovementType::In,
            None,
            dec!(50),
            Some(dec!(10.00)),
            Currency::usd(),
            dec!(50),
            None,
            None,
            actor_id,
            None,
        );
        
        assert_eq!(movement.total_cost(), Some(dec!(500.00)));
    }

    #[test]
    fn test_total_cost_with_negative_quantity() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let movement = InventoryMovement::create(
            stock_id,
            MovementType::Out,
            None,
            dec!(-30),
            Some(dec!(10.00)),
            Currency::usd(),
            dec!(70),
            None,
            None,
            actor_id,
            None,
        );
        
        assert_eq!(movement.total_cost(), Some(dec!(300.00)));
    }

    #[test]
    fn test_total_cost_without_unit_cost() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let movement = InventoryMovement::create(
            stock_id,
            MovementType::Adjustment,
            Some("correction".to_string()),
            dec!(10),
            None,
            Currency::usd(),
            dec!(110),
            None,
            None,
            actor_id,
            None,
        );
        
        assert!(movement.total_cost().is_none());
    }

    #[test]
    fn test_movement_types() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let types = [
            MovementType::In,
            MovementType::Out,
            MovementType::Adjustment,
            MovementType::TransferOut,
            MovementType::TransferIn,
            MovementType::Reservation,
            MovementType::Release,
        ];
        
        for movement_type in types {
            let movement = InventoryMovement::create(
                stock_id,
                movement_type,
                None,
                dec!(10),
                None,
                Currency::usd(),
                dec!(100),
                None,
                None,
                actor_id,
                None,
            );
            assert_eq!(movement.movement_type(), movement_type);
        }
    }

    #[test]
    fn test_movement_with_reference() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        let reference_id = new_uuid();
        
        let movement = InventoryMovement::create(
            stock_id,
            MovementType::Out,
            Some("sale".to_string()),
            dec!(-5),
            Some(dec!(15.00)),
            Currency::usd(),
            dec!(95),
            Some("order".to_string()),
            Some(reference_id),
            actor_id,
            None,
        );
        
        assert_eq!(movement.reference_type(), Some("order"));
        assert_eq!(movement.reference_id(), Some(reference_id));
    }

    #[test]
    fn test_reconstitute() {
        let id = MovementId::new();
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        let now = Utc::now();
        let metadata = serde_json::json!({"source": "import"});
        
        let movement = InventoryMovement::reconstitute(
            id,
            stock_id,
            MovementType::In,
            Some("import".to_string()),
            dec!(200),
            Some(dec!(5.00)),
            Currency::hnl(),
            dec!(200),
            Some("import".to_string()),
            Some(new_uuid()),
            actor_id,
            Some("Bulk import".to_string()),
            metadata.clone(),
            now,
        );
        
        assert_eq!(movement.id(), id);
        assert_eq!(movement.stock_id(), stock_id);
        assert_eq!(movement.movement_type(), MovementType::In);
        assert_eq!(movement.metadata(), &metadata);
        assert_eq!(movement.created_at(), now);
    }

    #[test]
    fn test_currency() {
        let stock_id = StockId::new();
        let actor_id = UserId::new();
        
        let movement_usd = InventoryMovement::create(
            stock_id,
            MovementType::In,
            None,
            dec!(10),
            Some(dec!(100.00)),
            Currency::usd(),
            dec!(10),
            None,
            None,
            actor_id,
            None,
        );
        assert_eq!(movement_usd.currency().as_str(), "USD");
        
        let movement_hnl = InventoryMovement::create(
            stock_id,
            MovementType::In,
            None,
            dec!(10),
            Some(dec!(2500.00)),
            Currency::hnl(),
            dec!(10),
            None,
            None,
            actor_id,
            None,
        );
        assert_eq!(movement_hnl.currency().as_str(), "HNL");
    }
}
