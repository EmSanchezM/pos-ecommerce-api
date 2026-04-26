//! Shipment - the actual fulfillment record for a sale.
//!
//! Polymorphic on `method_type` (carried via `shipping_method_id`):
//!   - StorePickup       → uses pickup_code, pickup_ready_at, pickup_expires_at
//!   - OwnDelivery       → uses driver_id
//!   - ExternalDelivery  → uses delivery_provider_id
//!
//! State machine rules live in `can_transition_to` and the `mark_*` methods.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::ShippingError;
use crate::domain::value_objects::{
    DeliveryProviderId, DriverId, ShipmentId, ShipmentStatus, ShippingMethodId, ShippingMethodType,
};
use identity::StoreId;
use sales::SaleId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub struct Shipment {
    id: ShipmentId,
    store_id: StoreId,
    sale_id: SaleId,
    shipping_method_id: ShippingMethodId,
    method_type: ShippingMethodType,

    driver_id: Option<DriverId>,
    delivery_provider_id: Option<DeliveryProviderId>,

    pickup_code: Option<String>,
    pickup_ready_at: Option<DateTime<Utc>>,
    pickup_expires_at: Option<DateTime<Utc>>,
    picked_up_at: Option<DateTime<Utc>>,
    picked_up_by_name: Option<String>,

    requires_cash_collection: bool,
    cash_amount: Option<Decimal>,

    status: ShipmentStatus,
    tracking_number: Option<String>,
    carrier_name: Option<String>,
    shipping_cost: Decimal,
    currency: String,
    weight_kg: Option<Decimal>,

    recipient_name: String,
    recipient_phone: Option<String>,
    address_line1: String,
    address_line2: Option<String>,
    city: String,
    state: String,
    postal_code: Option<String>,
    country: String,

    notes: Option<String>,
    failure_reason: Option<String>,
    attempt_count: i32,

    shipped_at: Option<DateTime<Utc>>,
    delivered_at: Option<DateTime<Utc>>,
    estimated_delivery: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    cancel_reason: Option<String>,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Shipment {
    /// Creates a new shipment in `Pending` status.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        sale_id: SaleId,
        shipping_method_id: ShippingMethodId,
        method_type: ShippingMethodType,
        shipping_cost: Decimal,
        currency: String,
        weight_kg: Option<Decimal>,
        recipient_name: String,
        recipient_phone: Option<String>,
        address_line1: String,
        address_line2: Option<String>,
        city: String,
        state: String,
        postal_code: Option<String>,
        country: String,
        notes: Option<String>,
        requires_cash_collection: bool,
        cash_amount: Option<Decimal>,
    ) -> Result<Self, ShippingError> {
        if shipping_cost < Decimal::ZERO {
            return Err(ShippingError::InvalidAmount);
        }
        if requires_cash_collection {
            match cash_amount {
                Some(a) if a > Decimal::ZERO => {}
                _ => return Err(ShippingError::InvalidAmount),
            }
        }
        let now = Utc::now();
        Ok(Self {
            id: ShipmentId::new(),
            store_id,
            sale_id,
            shipping_method_id,
            method_type,
            driver_id: None,
            delivery_provider_id: None,
            pickup_code: None,
            pickup_ready_at: None,
            pickup_expires_at: None,
            picked_up_at: None,
            picked_up_by_name: None,
            requires_cash_collection,
            cash_amount,
            status: ShipmentStatus::Pending,
            tracking_number: None,
            carrier_name: None,
            shipping_cost,
            currency,
            weight_kg,
            recipient_name,
            recipient_phone,
            address_line1,
            address_line2,
            city,
            state,
            postal_code,
            country,
            notes,
            failure_reason: None,
            attempt_count: 0,
            shipped_at: None,
            delivered_at: None,
            estimated_delivery: None,
            cancelled_at: None,
            cancel_reason: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShipmentId,
        store_id: StoreId,
        sale_id: SaleId,
        shipping_method_id: ShippingMethodId,
        method_type: ShippingMethodType,
        driver_id: Option<DriverId>,
        delivery_provider_id: Option<DeliveryProviderId>,
        pickup_code: Option<String>,
        pickup_ready_at: Option<DateTime<Utc>>,
        pickup_expires_at: Option<DateTime<Utc>>,
        picked_up_at: Option<DateTime<Utc>>,
        picked_up_by_name: Option<String>,
        requires_cash_collection: bool,
        cash_amount: Option<Decimal>,
        status: ShipmentStatus,
        tracking_number: Option<String>,
        carrier_name: Option<String>,
        shipping_cost: Decimal,
        currency: String,
        weight_kg: Option<Decimal>,
        recipient_name: String,
        recipient_phone: Option<String>,
        address_line1: String,
        address_line2: Option<String>,
        city: String,
        state: String,
        postal_code: Option<String>,
        country: String,
        notes: Option<String>,
        failure_reason: Option<String>,
        attempt_count: i32,
        shipped_at: Option<DateTime<Utc>>,
        delivered_at: Option<DateTime<Utc>>,
        estimated_delivery: Option<DateTime<Utc>>,
        cancelled_at: Option<DateTime<Utc>>,
        cancel_reason: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            sale_id,
            shipping_method_id,
            method_type,
            driver_id,
            delivery_provider_id,
            pickup_code,
            pickup_ready_at,
            pickup_expires_at,
            picked_up_at,
            picked_up_by_name,
            requires_cash_collection,
            cash_amount,
            status,
            tracking_number,
            carrier_name,
            shipping_cost,
            currency,
            weight_kg,
            recipient_name,
            recipient_phone,
            address_line1,
            address_line2,
            city,
            state,
            postal_code,
            country,
            notes,
            failure_reason,
            attempt_count,
            shipped_at,
            delivered_at,
            estimated_delivery,
            cancelled_at,
            cancel_reason,
            created_at,
            updated_at,
        }
    }

    // -------------------------------------------------------------------------
    // State transitions
    // -------------------------------------------------------------------------

    /// True when `to` is reachable from the current status under this method's
    /// lifecycle. Cancel is allowed from any non-terminal status.
    pub fn can_transition_to(&self, to: ShipmentStatus) -> bool {
        if self.status.is_terminal() {
            return false;
        }
        if matches!(to, ShipmentStatus::Cancelled) {
            return true;
        }
        use ShipmentStatus::*;
        match (self.method_type, self.status, to) {
            // StorePickup
            (ShippingMethodType::StorePickup, Pending, ReadyForPickup) => true,
            (ShippingMethodType::StorePickup, ReadyForPickup, PickedUp) => true,
            (ShippingMethodType::StorePickup, ReadyForPickup, Expired) => true,
            // OwnDelivery
            (ShippingMethodType::OwnDelivery, Pending, Assigned) => true,
            (ShippingMethodType::OwnDelivery, Assigned, InTransit) => true,
            (ShippingMethodType::OwnDelivery, InTransit, OutForDelivery) => true,
            (ShippingMethodType::OwnDelivery, InTransit, Delivered) => true,
            (ShippingMethodType::OwnDelivery, OutForDelivery, Delivered) => true,
            (ShippingMethodType::OwnDelivery, OutForDelivery, Failed) => true,
            (ShippingMethodType::OwnDelivery, InTransit, Failed) => true,
            (ShippingMethodType::OwnDelivery, Failed, Assigned) => true, // reschedule
            (ShippingMethodType::OwnDelivery, Failed, Returned) => true,
            // ExternalDelivery
            (ShippingMethodType::ExternalDelivery, Pending, Dispatched) => true,
            (ShippingMethodType::ExternalDelivery, Dispatched, InTransit) => true,
            (ShippingMethodType::ExternalDelivery, InTransit, OutForDelivery) => true,
            (ShippingMethodType::ExternalDelivery, InTransit, Delivered) => true,
            (ShippingMethodType::ExternalDelivery, OutForDelivery, Delivered) => true,
            (ShippingMethodType::ExternalDelivery, OutForDelivery, Failed) => true,
            (ShippingMethodType::ExternalDelivery, InTransit, Failed) => true,
            (ShippingMethodType::ExternalDelivery, Failed, Returned) => true,
            // Generic methods follow OwnDelivery rules (manual driving)
            (
                ShippingMethodType::Standard
                | ShippingMethodType::Express
                | ShippingMethodType::SameDay
                | ShippingMethodType::FreeShipping,
                from,
                to,
            ) => matches!(
                (from, to),
                (Pending, Dispatched)
                    | (Pending, InTransit)
                    | (Dispatched, InTransit)
                    | (InTransit, OutForDelivery)
                    | (InTransit, Delivered)
                    | (OutForDelivery, Delivered)
                    | (OutForDelivery, Failed)
                    | (Failed, Returned)
            ),
            _ => false,
        }
    }

    /// Mark the shipment ready for pickup (StorePickup only). Auto-generates
    /// a 6-digit pickup code and sets an expiration window.
    pub fn mark_ready_for_pickup(&mut self, pickup_window: Duration) -> Result<(), ShippingError> {
        if !matches!(self.method_type, ShippingMethodType::StorePickup) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::ReadyForPickup.to_string(),
            });
        }
        if !self.can_transition_to(ShipmentStatus::ReadyForPickup) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::ReadyForPickup.to_string(),
            });
        }
        let now = Utc::now();
        self.status = ShipmentStatus::ReadyForPickup;
        if self.pickup_code.is_none() {
            self.pickup_code = Some(generate_pickup_code());
        }
        self.pickup_ready_at = Some(now);
        self.pickup_expires_at = Some(now + pickup_window);
        self.touch();
        Ok(())
    }

    /// Confirm pickup at the counter. `picked_up_by_name` is whoever showed up.
    pub fn mark_picked_up(&mut self, picked_up_by_name: String) -> Result<(), ShippingError> {
        if !self.can_transition_to(ShipmentStatus::PickedUp) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::PickedUp.to_string(),
            });
        }
        if let Some(exp) = self.pickup_expires_at
            && Utc::now() > exp
        {
            return Err(ShippingError::PickupExpired);
        }
        let now = Utc::now();
        self.status = ShipmentStatus::PickedUp;
        self.picked_up_at = Some(now);
        self.picked_up_by_name = Some(picked_up_by_name);
        self.delivered_at = Some(now);
        self.touch();
        Ok(())
    }

    /// Auto-expire a pickup whose window passed without being claimed.
    pub fn mark_expired(&mut self) -> Result<(), ShippingError> {
        if !self.can_transition_to(ShipmentStatus::Expired) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Expired.to_string(),
            });
        }
        self.status = ShipmentStatus::Expired;
        self.touch();
        Ok(())
    }

    /// Assign a driver (OwnDelivery). Caller must validate driver availability.
    pub fn assign_driver(&mut self, driver_id: DriverId) -> Result<(), ShippingError> {
        if !self.method_type.requires_driver() {
            return Err(ShippingError::DriverAssignmentNotAllowed);
        }
        if !self.can_transition_to(ShipmentStatus::Assigned) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Assigned.to_string(),
            });
        }
        self.driver_id = Some(driver_id);
        self.status = ShipmentStatus::Assigned;
        self.touch();
        Ok(())
    }

    /// Reassign a driver after a Failed attempt — bumps `attempt_count`.
    pub fn reschedule_with_driver(&mut self, driver_id: DriverId) -> Result<(), ShippingError> {
        if !self.method_type.requires_driver() {
            return Err(ShippingError::DriverAssignmentNotAllowed);
        }
        if !matches!(self.status, ShipmentStatus::Failed) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Assigned.to_string(),
            });
        }
        self.driver_id = Some(driver_id);
        self.status = ShipmentStatus::Assigned;
        self.attempt_count += 1;
        self.failure_reason = None;
        self.touch();
        Ok(())
    }

    /// Hand off to an external provider (ExternalDelivery).
    pub fn dispatch_to_provider(
        &mut self,
        provider_id: DeliveryProviderId,
        tracking_number: Option<String>,
        carrier_name: Option<String>,
        estimated_delivery: Option<DateTime<Utc>>,
    ) -> Result<(), ShippingError> {
        if !self.method_type.requires_provider() {
            return Err(ShippingError::ProviderAssignmentNotAllowed);
        }
        if !self.can_transition_to(ShipmentStatus::Dispatched) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Dispatched.to_string(),
            });
        }
        self.delivery_provider_id = Some(provider_id);
        self.tracking_number = tracking_number;
        self.carrier_name = carrier_name;
        self.estimated_delivery = estimated_delivery;
        self.status = ShipmentStatus::Dispatched;
        self.shipped_at = Some(Utc::now());
        self.touch();
        Ok(())
    }

    /// Generic status update for `InTransit` / `OutForDelivery`.
    pub fn transition_to(&mut self, to: ShipmentStatus) -> Result<(), ShippingError> {
        if !self.can_transition_to(to) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: to.to_string(),
            });
        }
        if matches!(to, ShipmentStatus::InTransit) && self.shipped_at.is_none() {
            self.shipped_at = Some(Utc::now());
        }
        self.status = to;
        self.touch();
        Ok(())
    }

    pub fn mark_delivered(&mut self) -> Result<(), ShippingError> {
        if !self.can_transition_to(ShipmentStatus::Delivered) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Delivered.to_string(),
            });
        }
        let now = Utc::now();
        self.status = ShipmentStatus::Delivered;
        self.delivered_at = Some(now);
        if self.shipped_at.is_none() {
            self.shipped_at = Some(now);
        }
        self.touch();
        Ok(())
    }

    pub fn mark_failed(&mut self, reason: String) -> Result<(), ShippingError> {
        if !self.can_transition_to(ShipmentStatus::Failed) {
            return Err(ShippingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: ShipmentStatus::Failed.to_string(),
            });
        }
        self.status = ShipmentStatus::Failed;
        self.failure_reason = Some(reason);
        self.attempt_count += 1;
        self.touch();
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), ShippingError> {
        if self.status.is_terminal() {
            return Err(ShippingError::ShipmentAlreadyDelivered);
        }
        self.status = ShipmentStatus::Cancelled;
        self.cancelled_at = Some(Utc::now());
        self.cancel_reason = Some(reason);
        self.touch();
        Ok(())
    }

    pub fn set_tracking(&mut self, tracking_number: String, carrier_name: Option<String>) {
        self.tracking_number = Some(tracking_number);
        self.carrier_name = carrier_name;
        self.touch();
    }

    pub fn set_estimated_delivery(&mut self, eta: Option<DateTime<Utc>>) {
        self.estimated_delivery = eta;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    pub fn id(&self) -> ShipmentId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn sale_id(&self) -> SaleId {
        self.sale_id
    }
    pub fn shipping_method_id(&self) -> ShippingMethodId {
        self.shipping_method_id
    }
    pub fn method_type(&self) -> ShippingMethodType {
        self.method_type
    }
    pub fn driver_id(&self) -> Option<DriverId> {
        self.driver_id
    }
    pub fn delivery_provider_id(&self) -> Option<DeliveryProviderId> {
        self.delivery_provider_id
    }
    pub fn pickup_code(&self) -> Option<&str> {
        self.pickup_code.as_deref()
    }
    pub fn pickup_ready_at(&self) -> Option<DateTime<Utc>> {
        self.pickup_ready_at
    }
    pub fn pickup_expires_at(&self) -> Option<DateTime<Utc>> {
        self.pickup_expires_at
    }
    pub fn picked_up_at(&self) -> Option<DateTime<Utc>> {
        self.picked_up_at
    }
    pub fn picked_up_by_name(&self) -> Option<&str> {
        self.picked_up_by_name.as_deref()
    }
    pub fn requires_cash_collection(&self) -> bool {
        self.requires_cash_collection
    }
    pub fn cash_amount(&self) -> Option<Decimal> {
        self.cash_amount
    }
    pub fn status(&self) -> ShipmentStatus {
        self.status
    }
    pub fn tracking_number(&self) -> Option<&str> {
        self.tracking_number.as_deref()
    }
    pub fn carrier_name(&self) -> Option<&str> {
        self.carrier_name.as_deref()
    }
    pub fn shipping_cost(&self) -> Decimal {
        self.shipping_cost
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn weight_kg(&self) -> Option<Decimal> {
        self.weight_kg
    }
    pub fn recipient_name(&self) -> &str {
        &self.recipient_name
    }
    pub fn recipient_phone(&self) -> Option<&str> {
        self.recipient_phone.as_deref()
    }
    pub fn address_line1(&self) -> &str {
        &self.address_line1
    }
    pub fn address_line2(&self) -> Option<&str> {
        self.address_line2.as_deref()
    }
    pub fn city(&self) -> &str {
        &self.city
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn postal_code(&self) -> Option<&str> {
        self.postal_code.as_deref()
    }
    pub fn country(&self) -> &str {
        &self.country
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }
    pub fn attempt_count(&self) -> i32 {
        self.attempt_count
    }
    pub fn shipped_at(&self) -> Option<DateTime<Utc>> {
        self.shipped_at
    }
    pub fn delivered_at(&self) -> Option<DateTime<Utc>> {
        self.delivered_at
    }
    pub fn estimated_delivery(&self) -> Option<DateTime<Utc>> {
        self.estimated_delivery
    }
    pub fn cancelled_at(&self) -> Option<DateTime<Utc>> {
        self.cancelled_at
    }
    pub fn cancel_reason(&self) -> Option<&str> {
        self.cancel_reason.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

/// 6-digit pickup code derived from the bottom 24 bits of a fresh UUID v7.
/// Not cryptographically random — fine for an over-the-counter check.
fn generate_pickup_code() -> String {
    let bytes = Uuid::new_v7(Timestamp::now(NoContext)).into_bytes();
    let n = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) % 1_000_000;
    format!("{:06}", n)
}
