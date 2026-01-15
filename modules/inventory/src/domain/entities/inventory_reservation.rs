// InventoryReservation entity - temporary stock holds for shopping carts

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{ReservationId, ReservationStatus, StockId};
use crate::InventoryError;

/// InventoryReservation entity representing a temporary hold on stock.
/// Used for shopping carts, orders, and quotes to prevent overselling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryReservation {
    id: ReservationId,
    stock_id: StockId,
    reference_type: String,
    reference_id: Uuid,
    quantity: Decimal,
    status: ReservationStatus,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl InventoryReservation {
    /// Creates a new pending reservation
    pub fn create(
        stock_id: StockId,
        reference_type: String,
        reference_id: Uuid,
        quantity: Decimal,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        if expires_at <= Utc::now() {
            return Err(InventoryError::ReservationExpired);
        }
        
        let now = Utc::now();
        Ok(Self {
            id: ReservationId::new(),
            stock_id,
            reference_type,
            reference_id,
            quantity,
            status: ReservationStatus::Pending,
            expires_at,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes an InventoryReservation from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ReservationId,
        stock_id: StockId,
        reference_type: String,
        reference_id: Uuid,
        quantity: Decimal,
        status: ReservationStatus,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            stock_id,
            reference_type,
            reference_id,
            quantity,
            status,
            expires_at,
            created_at,
            updated_at,
        }
    }

    /// Confirms the reservation (e.g., when order is placed)
    pub fn confirm(&mut self) -> Result<(), InventoryError> {
        if self.status != ReservationStatus::Pending {
            return Err(InventoryError::InvalidReservationStatus);
        }
        self.status = ReservationStatus::Confirmed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancels the reservation
    pub fn cancel(&mut self) -> Result<(), InventoryError> {
        if self.status != ReservationStatus::Pending {
            return Err(InventoryError::InvalidReservationStatus);
        }
        self.status = ReservationStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marks the reservation as expired
    pub fn expire(&mut self) -> Result<(), InventoryError> {
        if self.status != ReservationStatus::Pending {
            return Err(InventoryError::InvalidReservationStatus);
        }
        self.status = ReservationStatus::Expired;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns true if the reservation has expired (based on current time)
    pub fn is_expired(&self) -> bool {
        self.status == ReservationStatus::Pending && Utc::now() > self.expires_at
    }

    /// Returns true if the reservation is still active (pending and not expired)
    pub fn is_active(&self) -> bool {
        self.status == ReservationStatus::Pending && !self.is_expired()
    }

    pub fn id(&self) -> ReservationId { self.id }
    pub fn stock_id(&self) -> StockId { self.stock_id }
    pub fn reference_type(&self) -> &str { &self.reference_type }
    pub fn reference_id(&self) -> Uuid { self.reference_id }
    pub fn quantity(&self) -> Decimal { self.quantity }
    pub fn status(&self) -> ReservationStatus { self.status }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn future_time() -> DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[test]
    fn test_create_reservation() {
        let stock_id = StockId::new();
        let reference_id = new_uuid();
        let expires_at = future_time();
        
        let reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            reference_id,
            dec!(5),
            expires_at,
        ).unwrap();
        
        assert_eq!(reservation.stock_id(), stock_id);
        assert_eq!(reservation.reference_type(), "cart");
        assert_eq!(reservation.quantity(), dec!(5));
        assert_eq!(reservation.status(), ReservationStatus::Pending);
    }

    #[test]
    fn test_confirm_reservation() {
        let stock_id = StockId::new();
        let mut reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(5),
            future_time(),
        ).unwrap();
        
        reservation.confirm().unwrap();
        assert_eq!(reservation.status(), ReservationStatus::Confirmed);
    }

    #[test]
    fn test_cancel_reservation() {
        let stock_id = StockId::new();
        let mut reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(5),
            future_time(),
        ).unwrap();
        
        reservation.cancel().unwrap();
        assert_eq!(reservation.status(), ReservationStatus::Cancelled);
    }

    #[test]
    fn test_expire_reservation() {
        let stock_id = StockId::new();
        let mut reservation = InventoryReservation::create(
            stock_id,
            "cart".to_string(),
            new_uuid(),
            dec!(5),
            future_time(),
        ).unwrap();
        
        reservation.expire().unwrap();
        assert_eq!(reservation.status(), ReservationStatus::Expired);
    }
}
