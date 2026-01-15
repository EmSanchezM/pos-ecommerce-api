// CreateReservationUseCase - creates a new inventory reservation

use std::sync::Arc;

use chrono::Utc;

use crate::application::dtos::commands::CreateReservationCommand;
use crate::application::dtos::responses::ReservationResponse;
use crate::domain::entities::InventoryReservation;
use crate::domain::repositories::{InventoryStockRepository, ReservationRepository};
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Use case for creating an inventory reservation.
///
/// Validates stock has sufficient available quantity, validates expires_at is in future,
/// and increases reserved_quantity on stock.
pub struct CreateReservationUseCase<S, R>
where
    S: InventoryStockRepository,
    R: ReservationRepository,
{
    stock_repo: Arc<S>,
    reservation_repo: Arc<R>,
}

impl<S, R> CreateReservationUseCase<S, R>
where
    S: InventoryStockRepository,
    R: ReservationRepository,
{
    /// Creates a new instance of CreateReservationUseCase
    pub fn new(stock_repo: Arc<S>, reservation_repo: Arc<R>) -> Self {
        Self {
            stock_repo,
            reservation_repo,
        }
    }

    /// Executes the use case to create a new reservation
    ///
    /// # Arguments
    /// * `command` - The create reservation command containing reservation data
    ///
    /// # Returns
    /// ReservationResponse on success
    ///
    /// # Errors
    /// * `InventoryError::StockNotFound` - If stock record doesn't exist
    /// * `InventoryError::InsufficientStock` - If not enough available quantity
    /// * `InventoryError::ReservationExpired` - If expires_at is not in the future
    /// * `InventoryError::OptimisticLockError` - If concurrent modification detected
    pub async fn execute(
        &self,
        command: CreateReservationCommand,
    ) -> Result<ReservationResponse, InventoryError> {
        // 1. Validate expires_at is in future (Requirement 4.3)
        if command.expires_at <= Utc::now() {
            return Err(InventoryError::ReservationExpired);
        }

        // 2. Find stock record
        let stock_id = StockId::from_uuid(command.stock_id);
        let mut stock = self
            .stock_repo
            .find_by_id(stock_id)
            .await?
            .ok_or(InventoryError::StockNotFound(command.stock_id))?;

        // 3. Validate sufficient available quantity (Requirement 4.1)
        if command.quantity > stock.available_quantity() {
            return Err(InventoryError::InsufficientStock);
        }

        // 4. Create reservation entity
        let reservation = InventoryReservation::create(
            stock_id,
            command.reference_type,
            command.reference_id,
            command.quantity,
            command.expires_at,
        )?;

        // 5. Increase reserved_quantity on stock (Requirement 4.1)
        let expected_version = stock.version();
        stock.reserve(command.quantity)?;
        stock.increment_version();

        // 6. Update stock with optimistic locking
        self.stock_repo
            .update_with_version(&stock, expected_version)
            .await?;

        // 7. Save reservation
        self.reservation_repo.save(&reservation).await?;

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
    use chrono::Duration;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::InventoryStock;
    use crate::domain::value_objects::{ProductId, ReservationId};
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    // Mock repositories for testing
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
    }

    struct MockReservationRepository {
        reservations: Mutex<HashMap<ReservationId, InventoryReservation>>,
    }

    impl MockReservationRepository {
        fn new() -> Self {
            Self {
                reservations: Mutex::new(HashMap::new()),
            }
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
    }

    fn future_time() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    #[tokio::test]
    async fn test_create_reservation_success() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let reservation_repo = Arc::new(MockReservationRepository::new());

        // Create initial stock with 100 units
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        let use_case = CreateReservationUseCase::new(stock_repo.clone(), reservation_repo.clone());

        let command = CreateReservationCommand {
            stock_id: stock_id.into_uuid(),
            reference_type: "cart".to_string(),
            reference_id: new_uuid(),
            quantity: dec!(30),
            expires_at: future_time(),
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.quantity, dec!(30));
        assert_eq!(response.status, "pending");
        assert_eq!(response.reference_type, "cart");

        // Verify stock was updated
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.reserved_quantity(), dec!(30));
        assert_eq!(updated_stock.available_quantity(), dec!(70));
    }

    #[tokio::test]
    async fn test_create_reservation_insufficient_stock() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let reservation_repo = Arc::new(MockReservationRepository::new());

        // Create initial stock with 50 units
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(50)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        let use_case = CreateReservationUseCase::new(stock_repo, reservation_repo);

        // Try to reserve more than available
        let command = CreateReservationCommand {
            stock_id: stock_id.into_uuid(),
            reference_type: "cart".to_string(),
            reference_id: new_uuid(),
            quantity: dec!(100),
            expires_at: future_time(),
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(InventoryError::InsufficientStock)));
    }

    #[tokio::test]
    async fn test_create_reservation_expired_time() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let reservation_repo = Arc::new(MockReservationRepository::new());

        // Create initial stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        let use_case = CreateReservationUseCase::new(stock_repo, reservation_repo);

        // Use past expiration time
        let command = CreateReservationCommand {
            stock_id: stock_id.into_uuid(),
            reference_type: "cart".to_string(),
            reference_id: new_uuid(),
            quantity: dec!(30),
            expires_at: Utc::now() - Duration::hours(1),
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(InventoryError::ReservationExpired)));
    }

    #[tokio::test]
    async fn test_create_reservation_stock_not_found() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let reservation_repo = Arc::new(MockReservationRepository::new());

        let use_case = CreateReservationUseCase::new(stock_repo, reservation_repo);

        let command = CreateReservationCommand {
            stock_id: new_uuid(),
            reference_type: "cart".to_string(),
            reference_id: new_uuid(),
            quantity: dec!(30),
            expires_at: future_time(),
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(InventoryError::StockNotFound(_))));
    }
}
