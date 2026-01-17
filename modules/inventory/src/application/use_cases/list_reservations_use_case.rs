// ListReservationsUseCase - lists reservations with pagination and filters

use std::sync::Arc;

use uuid::Uuid;

use crate::application::dtos::responses::{PaginatedResponse, ReservationResponse};
use crate::domain::repositories::ReservationRepository;
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Query parameters for listing reservations
#[derive(Debug, Clone)]
pub struct ListReservationsQuery {
    /// Optional filter by stock ID
    pub stock_id: Option<Uuid>,
    /// Optional filter by status (pending, confirmed, cancelled, expired)
    pub status: Option<String>,
    /// Optional filter by reference type (cart, order, quote)
    pub reference_type: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

/// Use case for listing inventory reservations with pagination and filters.
pub struct ListReservationsUseCase<R>
where
    R: ReservationRepository,
{
    reservation_repo: Arc<R>,
}

impl<R> ListReservationsUseCase<R>
where
    R: ReservationRepository,
{
    /// Creates a new instance of ListReservationsUseCase
    pub fn new(reservation_repo: Arc<R>) -> Self {
        Self { reservation_repo }
    }

    /// Executes the use case to list reservations
    ///
    /// # Arguments
    /// * `query` - The query parameters for filtering and pagination
    ///
    /// # Returns
    /// PaginatedResponse of ReservationResponse on success
    pub async fn execute(
        &self,
        query: ListReservationsQuery,
    ) -> Result<PaginatedResponse<ReservationResponse>, InventoryError> {
        // Validate pagination parameters
        let page = if query.page < 1 { 1 } else { query.page };
        let page_size = query.page_size.clamp(1, 100);

        // Convert stock_id to domain type if present
        let stock_id = query.stock_id.map(StockId::from_uuid);

        // Find reservations with pagination
        let (reservations, total_count) = self
            .reservation_repo
            .find_paginated(
                stock_id,
                query.status.as_deref(),
                query.reference_type.as_deref(),
                page,
                page_size,
            )
            .await?;

        // Convert to response DTOs
        let responses: Vec<ReservationResponse> = reservations
            .into_iter()
            .map(|r| ReservationResponse {
                id: r.id().into_uuid(),
                stock_id: r.stock_id().into_uuid(),
                reference_type: r.reference_type().to_string(),
                reference_id: r.reference_id(),
                quantity: r.quantity(),
                status: r.status().to_string(),
                expires_at: r.expires_at(),
                created_at: r.created_at(),
                updated_at: r.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(responses, page, page_size, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::InventoryReservation;
    use crate::domain::value_objects::ReservationId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
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
            stock_id: Option<StockId>,
            status: Option<&str>,
            reference_type: Option<&str>,
            page: i64,
            page_size: i64,
        ) -> Result<(Vec<InventoryReservation>, i64), InventoryError> {
            let reservations = self.reservations.lock().unwrap();
            let mut filtered: Vec<_> = reservations
                .values()
                .filter(|r| {
                    if let Some(sid) = stock_id {
                        if r.stock_id() != sid {
                            return false;
                        }
                    }
                    if let Some(s) = status {
                        if r.status().to_string() != s {
                            return false;
                        }
                    }
                    if let Some(rt) = reference_type {
                        if r.reference_type() != rt {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();

            let total = filtered.len() as i64;
            let offset = ((page - 1) * page_size) as usize;
            let result: Vec<_> = filtered
                .drain(..)
                .skip(offset)
                .take(page_size as usize)
                .collect();

            Ok((result, total))
        }
    }

    fn future_time() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    #[tokio::test]
    async fn test_list_reservations_success() {
        let reservation_repo = Arc::new(MockReservationRepository::new());

        // Create some reservations
        let stock_id = StockId::new();
        for _ in 0..5 {
            let reservation = InventoryReservation::create(
                stock_id,
                "cart".to_string(),
                new_uuid(),
                dec!(10),
                future_time(),
            )
            .unwrap();
            reservation_repo.add_reservation(reservation);
        }

        let use_case = ListReservationsUseCase::new(reservation_repo);

        let query = ListReservationsQuery {
            stock_id: None,
            status: None,
            reference_type: None,
            page: 1,
            page_size: 10,
        };

        let result = use_case.execute(query).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.total_items, 5);
    }

    #[tokio::test]
    async fn test_list_reservations_with_status_filter() {
        let reservation_repo = Arc::new(MockReservationRepository::new());

        let stock_id = StockId::new();
        let reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(10),
            future_time(),
        )
        .unwrap();
        reservation_repo.add_reservation(reservation);

        let use_case = ListReservationsUseCase::new(reservation_repo);

        let query = ListReservationsQuery {
            stock_id: None,
            status: Some("pending".to_string()),
            reference_type: None,
            page: 1,
            page_size: 10,
        };

        let result = use_case.execute(query).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].status, "pending");
    }
}
