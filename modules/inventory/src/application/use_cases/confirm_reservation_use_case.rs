// ConfirmReservationUseCase - confirms a pending reservation

use std::sync::Arc;

use crate::application::dtos::commands::ConfirmReservationCommand;
use crate::application::dtos::responses::ReservationResponse;
use crate::domain::entities::InventoryMovement;
use crate::domain::repositories::{
    InventoryMovementRepository, InventoryStockRepository, ReservationRepository,
};
use crate::domain::value_objects::{Currency, MovementType, ReservationId};
use crate::InventoryError;
use identity::domain::value_objects::UserId;

/// Use case for confirming an inventory reservation.
///
/// Changes status to confirmed, creates inventory movement (out),
/// and decreases quantity on stock.
///
pub struct ConfirmReservationUseCase<R, S, M>
where
    R: ReservationRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    reservation_repo: Arc<R>,
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
}

impl<R, S, M> ConfirmReservationUseCase<R, S, M>
where
    R: ReservationRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    /// Creates a new instance of ConfirmReservationUseCase
    pub fn new(reservation_repo: Arc<R>, stock_repo: Arc<S>, movement_repo: Arc<M>) -> Self {
        Self {
            reservation_repo,
            stock_repo,
            movement_repo,
        }
    }

    /// Executes the use case to confirm a reservation
    ///
    /// # Arguments
    /// * `command` - The confirm reservation command
    /// * `actor_id` - ID of the user performing this action
    ///
    /// # Returns
    /// ReservationResponse on success
    ///
    /// # Errors
    /// * `InventoryError::ReservationNotFound` - If reservation doesn't exist
    /// * `InventoryError::InvalidReservationStatus` - If reservation is not pending
    /// * `InventoryError::StockNotFound` - If associated stock doesn't exist
    /// * `InventoryError::OptimisticLockError` - If concurrent modification detected
    pub async fn execute(
        &self,
        command: ConfirmReservationCommand,
        actor_id: UserId,
    ) -> Result<ReservationResponse, InventoryError> {
        // 1. Find reservation
        let reservation_id = ReservationId::from_uuid(command.reservation_id);
        let mut reservation = self
            .reservation_repo
            .find_by_id(reservation_id)
            .await?
            .ok_or(InventoryError::ReservationNotFound(command.reservation_id))?;

        // 2. Confirm reservation (validates status is pending)
        reservation.confirm()?;

        // 3. Find associated stock
        let mut stock = self
            .stock_repo
            .find_by_id(reservation.stock_id())
            .await?
            .ok_or(InventoryError::StockNotFound(
                reservation.stock_id().into_uuid(),
            ))?;

        // 4. Decrease quantity on stock (the reserved quantity becomes actual consumption)
        let expected_version = stock.version();
        let reserved_qty = reservation.quantity();

        // Release the reservation first
        stock.release(reserved_qty)?;
        // Then decrease the actual quantity
        stock.adjust_quantity(-reserved_qty)?;
        stock.increment_version();

        // 5. Update stock with optimistic locking
        self.stock_repo
            .update_with_version(&stock, expected_version)
            .await?;

        // 6. Create inventory movement (out)
        let movement = InventoryMovement::create(
            reservation.stock_id(),
            MovementType::Out,
            Some("reservation_confirmed".to_string()),
            -reserved_qty, // Negative for outgoing
            None,
            Currency::hnl(),
            stock.quantity(),
            Some(reservation.reference_type().to_string()),
            Some(reservation.reference_id()),
            actor_id,
            None,
        );
        self.movement_repo.save(&movement).await?;

        // 7. Update reservation
        self.reservation_repo.update(&reservation).await?;

        // 8. Convert to response
        Ok(ReservationResponse {
            id: reservation.id().into_uuid(),
            stock_id: reservation.stock_id().into_uuid(),
            reference_type: reservation.reference_type().to_string(),
            reference_id: reservation.reference_id(),
            quantity: reservation.quantity(),
            status: reservation.status().to_string(),
            expires_at: reservation.expires_at(),
            created_at: reservation.created_at(),
            updated_at: reservation.updated_at(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::{InventoryReservation, InventoryStock};
    use crate::domain::value_objects::{ProductId, StockId};
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    // Mock repositories
    struct MockReservationRepository {
        reservations: Mutex<HashMap<ReservationId, InventoryReservation>>,
    }

    impl MockReservationRepository {
        fn new() -> Self {
            Self {
                reservations: Mutex::new(HashMap::new()),
            }
        }

        fn add_reservation(&self, reservation: InventoryReservation) {
            let mut reservations = self.reservations.lock().unwrap();
            reservations.insert(reservation.id(), reservation);
        }
    }

    #[async_trait]
    impl ReservationRepository for MockReservationRepository {
        async fn save(&self, reservation: &InventoryReservation) -> Result<(), InventoryError> {
            let mut reservations = self.reservations.lock().unwrap();
            reservations.insert(reservation.id(), reservation.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: ReservationId,
        ) -> Result<Option<InventoryReservation>, InventoryError> {
            let reservations = self.reservations.lock().unwrap();
            Ok(reservations.get(&id).cloned())
        }

        async fn find_by_stock_id(
            &self,
            _stock_id: StockId,
        ) -> Result<Vec<InventoryReservation>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_reference(
            &self,
            _reference_type: &str,
            _reference_id: Uuid,
        ) -> Result<Vec<InventoryReservation>, InventoryError> {
            unimplemented!()
        }

        async fn find_expired(&self) -> Result<Vec<InventoryReservation>, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, reservation: &InventoryReservation) -> Result<(), InventoryError> {
            let mut reservations = self.reservations.lock().unwrap();
            reservations.insert(reservation.id(), reservation.clone());
            Ok(())
        }

        async fn delete(&self, _id: ReservationId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _stock_id: Option<StockId>,
            _status: Option<&str>,
            _reference_type: Option<&str>,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<InventoryReservation>, i64), InventoryError> {
            unimplemented!()
        }
    }

    struct MockStockRepository {
        stocks: Mutex<HashMap<StockId, InventoryStock>>,
    }

    impl MockStockRepository {
        fn new() -> Self {
            Self {
                stocks: Mutex::new(HashMap::new()),
            }
        }

        fn add_stock(&self, stock: InventoryStock) {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock);
        }
    }

    #[async_trait]
    impl InventoryStockRepository for MockStockRepository {
        async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.get(&id).cloned())
        }

        async fn find_by_store_and_product(
            &self,
            _store_id: StoreId,
            _product_id: ProductId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_store_and_variant(
            &self,
            _store_id: StoreId,
            _variant_id: crate::domain::value_objects::VariantId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn update_with_version(
            &self,
            stock: &InventoryStock,
            expected_version: i32,
        ) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            if let Some(existing) = stocks.get(&stock.id()) {
                if existing.version() != expected_version {
                    return Err(InventoryError::OptimisticLockError);
                }
                stocks.insert(stock.id(), stock.clone());
                Ok(())
            } else {
                Err(InventoryError::StockNotFound(stock.id().into_uuid()))
            }
        }

        async fn find_low_stock(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _store_id: Option<StoreId>,
            _product_id: Option<crate::domain::value_objects::ProductId>,
            _low_stock_only: bool,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<InventoryStock>, i64), InventoryError> {
            unimplemented!()
        }

        async fn find_by_product(
            &self,
            _product_id: crate::domain::value_objects::ProductId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }
    }

    struct MockMovementRepository {
        movements: Mutex<Vec<InventoryMovement>>,
    }

    impl MockMovementRepository {
        fn new() -> Self {
            Self {
                movements: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl InventoryMovementRepository for MockMovementRepository {
        async fn save(&self, movement: &InventoryMovement) -> Result<(), InventoryError> {
            let mut movements = self.movements.lock().unwrap();
            movements.push(movement.clone());
            Ok(())
        }

        async fn find_by_stock_id(
            &self,
            _stock_id: StockId,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_reference(
            &self,
            _reference_type: &str,
            _reference_id: Uuid,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            unimplemented!()
        }

        async fn calculate_weighted_average_cost(
            &self,
            _stock_id: StockId,
        ) -> Result<Option<Decimal>, InventoryError> {
            unimplemented!()
        }
    }

    fn future_time() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    #[tokio::test]
    async fn test_confirm_reservation_success() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create stock with 100 units, 30 reserved
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(30)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create pending reservation
        let reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(30),
            future_time(),
        )
        .unwrap();
        let reservation_id = reservation.id();
        reservation_repo.add_reservation(reservation);

        let use_case =
            ConfirmReservationUseCase::new(reservation_repo.clone(), stock_repo.clone(), movement_repo.clone());

        let command = ConfirmReservationCommand {
            reservation_id: reservation_id.into_uuid(),
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "confirmed");

        // Verify stock was updated
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.quantity(), dec!(70)); // 100 - 30
        assert_eq!(updated_stock.reserved_quantity(), dec!(0)); // Released

        // Verify movement was created
        let movements = movement_repo.movements.lock().unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].movement_type(), MovementType::Out);
        assert_eq!(movements[0].quantity(), dec!(-30));
    }

    #[tokio::test]
    async fn test_confirm_reservation_not_found() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let use_case = ConfirmReservationUseCase::new(reservation_repo, stock_repo, movement_repo);

        let command = ConfirmReservationCommand {
            reservation_id: new_uuid(),
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::ReservationNotFound(_))));
    }

    #[tokio::test]
    async fn test_confirm_already_confirmed_reservation() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create already confirmed reservation
        let mut reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(30),
            future_time(),
        )
        .unwrap();
        reservation.confirm().unwrap();
        let reservation_id = reservation.id();
        reservation_repo.add_reservation(reservation);

        let use_case = ConfirmReservationUseCase::new(reservation_repo, stock_repo, movement_repo);

        let command = ConfirmReservationCommand {
            reservation_id: reservation_id.into_uuid(),
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidReservationStatus)
        ));
    }
}
