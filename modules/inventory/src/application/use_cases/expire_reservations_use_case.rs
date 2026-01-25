// ExpireReservationsUseCase - batch process to expire pending reservations

use std::sync::Arc;

use crate::application::dtos::responses::ReservationResponse;
use crate::domain::repositories::{InventoryStockRepository, ReservationRepository};
use crate::InventoryError;

/// Result of the batch expiration process
#[derive(Debug, Clone)]
pub struct ExpireReservationsResult {
    /// Number of reservations successfully expired
    pub expired_count: usize,
    /// Number of reservations that failed to expire
    pub failed_count: usize,
    /// Details of expired reservations
    pub expired_reservations: Vec<ReservationResponse>,
    /// Errors encountered during processing
    pub errors: Vec<String>,
}

/// Use case for expiring pending reservations in batch.
///
/// Finds all expired pending reservations, marks them as expired,
/// and releases reserved quantities.
///
pub struct ExpireReservationsUseCase<R, S>
where
    R: ReservationRepository,
    S: InventoryStockRepository,
{
    reservation_repo: Arc<R>,
    stock_repo: Arc<S>,
}

impl<R, S> ExpireReservationsUseCase<R, S>
where
    R: ReservationRepository,
    S: InventoryStockRepository,
{
    /// Creates a new instance of ExpireReservationsUseCase
    pub fn new(reservation_repo: Arc<R>, stock_repo: Arc<S>) -> Self {
        Self {
            reservation_repo,
            stock_repo,
        }
    }

    /// Executes the use case to expire all pending reservations past their expiration time
    ///
    /// # Returns
    /// ExpireReservationsResult with counts and details of processed reservations
    ///
    /// # Notes
    /// This is a batch operation that processes all expired reservations.
    /// Individual failures are logged but don't stop the overall process.
    pub async fn execute(&self) -> Result<ExpireReservationsResult, InventoryError> {
        // 1. Find all expired pending reservations (Requirement 4.4)
        let expired_reservations = self.reservation_repo.find_expired().await?;

        let mut result = ExpireReservationsResult {
            expired_count: 0,
            failed_count: 0,
            expired_reservations: Vec::new(),
            errors: Vec::new(),
        };

        // 2. Process each expired reservation
        for mut reservation in expired_reservations {
            let reservation_id = reservation.id().into_uuid();

            // Try to expire the reservation
            match self.process_single_expiration(&mut reservation).await {
                Ok(response) => {
                    result.expired_count += 1;
                    result.expired_reservations.push(response);
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to expire reservation {}: {}",
                        reservation_id, e
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Process a single reservation expiration
    async fn process_single_expiration(
        &self,
        reservation: &mut crate::domain::entities::InventoryReservation,
    ) -> Result<ReservationResponse, InventoryError> {
        // 1. Mark reservation as expired
        reservation.expire()?;

        // 2. Find associated stock
        let mut stock = self
            .stock_repo
            .find_by_id(reservation.stock_id())
            .await?
            .ok_or(InventoryError::StockNotFound(
                reservation.stock_id().into_uuid(),
            ))?;

        // 3. Release reserved quantity
        let expected_version = stock.version();
        stock.release(reservation.quantity())?;
        stock.increment_version();

        // 4. Update stock with optimistic locking
        self.stock_repo
            .update_with_version(&stock, expected_version)
            .await?;

        // 5. Update reservation
        self.reservation_repo.update(reservation).await?;

        // 6. Return response
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
    use crate::domain::value_objects::{ProductId, ReservationId, ReservationStatus, StockId};
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
            let reservations = self.reservations.lock().unwrap();
            let now = Utc::now();
            Ok(reservations
                .values()
                .filter(|r| r.status() == ReservationStatus::Pending && r.expires_at() < now)
                .cloned()
                .collect())
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

        async fn find_all(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_all_low_stock(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_low_stock_by_store(&self, _store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }
    }

    fn past_time() -> chrono::DateTime<Utc> {
        Utc::now() - Duration::hours(1)
    }

    fn future_time() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    // Helper to create a reservation with a specific expiration time
    // We need to use reconstitute since create validates expires_at > now
    fn create_expired_reservation(
        stock_id: StockId,
        quantity: Decimal,
    ) -> InventoryReservation {
        InventoryReservation::reconstitute(
            ReservationId::new(),
            stock_id,
            "cart".to_string(),
            new_uuid(),
            quantity,
            ReservationStatus::Pending,
            past_time(),
            Utc::now() - Duration::hours(2),
            Utc::now() - Duration::hours(2),
        )
    }

    #[tokio::test]
    async fn test_expire_reservations_success() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());

        // Create stock with 100 units, 50 reserved (for 2 reservations)
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(50)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create 2 expired reservations
        let reservation1 = create_expired_reservation(stock_id, dec!(20));
        let reservation2 = create_expired_reservation(stock_id, dec!(30));
        reservation_repo.add_reservation(reservation1);
        reservation_repo.add_reservation(reservation2);

        let use_case = ExpireReservationsUseCase::new(reservation_repo.clone(), stock_repo.clone());

        let result = use_case.execute().await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.expired_count, 2);
        assert_eq!(result.failed_count, 0);
        assert!(result.errors.is_empty());

        // Verify stock was updated - all reserved quantity released
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.quantity(), dec!(100));
        assert_eq!(updated_stock.reserved_quantity(), dec!(0));
    }

    #[tokio::test]
    async fn test_expire_reservations_no_expired() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());

        // Create stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(30)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create non-expired reservation (future expiration)
        let reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(30),
            future_time(),
        )
        .unwrap();
        reservation_repo.add_reservation(reservation);

        let use_case = ExpireReservationsUseCase::new(reservation_repo, stock_repo.clone());

        let result = use_case.execute().await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.expired_count, 0);
        assert_eq!(result.failed_count, 0);

        // Verify stock was NOT updated
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.reserved_quantity(), dec!(30));
    }

    #[tokio::test]
    async fn test_expire_reservations_mixed() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());

        // Create stock with 100 units, 50 reserved
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(50)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create 1 expired and 1 non-expired reservation
        let expired_reservation = create_expired_reservation(stock_id, dec!(20));
        reservation_repo.add_reservation(expired_reservation);

        let active_reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(30),
            future_time(),
        )
        .unwrap();
        reservation_repo.add_reservation(active_reservation);

        let use_case = ExpireReservationsUseCase::new(reservation_repo, stock_repo.clone());

        let result = use_case.execute().await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.expired_count, 1);
        assert_eq!(result.failed_count, 0);

        // Verify only expired reservation's quantity was released
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.reserved_quantity(), dec!(30)); // 50 - 20 = 30
    }

    #[tokio::test]
    async fn test_expire_reservations_empty() {
        let reservation_repo = Arc::new(MockReservationRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());

        let use_case = ExpireReservationsUseCase::new(reservation_repo, stock_repo);

        let result = use_case.execute().await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.expired_count, 0);
        assert_eq!(result.failed_count, 0);
        assert!(result.expired_reservations.is_empty());
    }
}
