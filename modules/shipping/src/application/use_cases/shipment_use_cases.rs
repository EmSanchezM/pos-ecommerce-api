//! Shipment lifecycle use cases.
//!
//! Includes the COD bridge: when a shipment with `requires_cash_collection`
//! reaches `delivered`, the matching pending `payments::Transaction` for the
//! sale is auto-confirmed. Coupling lives at this seam (shipping depends on
//! payments, not the other way around).

use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{
    AssignDriverCommand, CancelShipmentCommand, ConfirmPickupCommand, CreateShipmentCommand,
    DispatchProviderCommand, ListShipmentsQuery, MarkDeliveredCommand, MarkFailedCommand,
    RescheduleShipmentCommand, ShipmentListResponse, ShipmentResponse, UpdateShipmentStatusCommand,
    UpdateTrackingCommand,
};
use crate::domain::entities::{Shipment, ShipmentTrackingEvent};
use crate::domain::repositories::{
    DeliveryProviderRepository, DriverRepository, ShipmentFilter, ShipmentRepository,
    ShipmentTrackingEventRepository, ShippingMethodRepository,
};
use crate::domain::value_objects::{
    DeliveryProviderId, DriverId, DriverStatus, ShipmentId, ShipmentStatus, ShippingMethodId,
    ShippingMethodType, TrackingEventSource,
};
use crate::infrastructure::adapters::{DeliveryProviderRegistry, DispatchRequest};
use identity::{StoreId, UserId};
use sales::SaleId;

const DEFAULT_PICKUP_WINDOW_HOURS: i64 = 72;

/// Shared dependencies for the shipment lifecycle use cases.
pub struct ShipmentDeps {
    pub method_repo: Arc<dyn ShippingMethodRepository>,
    pub driver_repo: Arc<dyn DriverRepository>,
    pub provider_repo: Arc<dyn DeliveryProviderRepository>,
    pub shipment_repo: Arc<dyn ShipmentRepository>,
    pub event_repo: Arc<dyn ShipmentTrackingEventRepository>,
    pub provider_registry: Arc<dyn DeliveryProviderRegistry>,
    pub transaction_repo: Arc<dyn payments::TransactionRepository>,
}

// =============================================================================
// CreateShipmentUseCase
// =============================================================================

pub struct CreateShipmentUseCase {
    deps: Arc<ShipmentDeps>,
}

impl CreateShipmentUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: CreateShipmentCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let sale_id = SaleId::from_uuid(cmd.sale_id);
        let method_id = ShippingMethodId::from_uuid(cmd.shipping_method_id);

        // Reject duplicate shipment for the same sale (DB has UNIQUE too).
        if self
            .deps
            .shipment_repo
            .find_by_sale_id(sale_id)
            .await?
            .is_some()
        {
            return Err(ShippingError::ShipmentAlreadyExistsForSale(cmd.sale_id));
        }

        let method = self.deps.method_repo.find_by_id(method_id).await?.ok_or(
            ShippingError::ShippingMethodNotFound(cmd.shipping_method_id),
        )?;

        let shipment = Shipment::create(
            store_id,
            sale_id,
            method_id,
            method.method_type(),
            cmd.shipping_cost,
            cmd.currency,
            cmd.weight_kg,
            cmd.recipient_name,
            cmd.recipient_phone,
            cmd.address_line1,
            cmd.address_line2,
            cmd.city,
            cmd.state,
            cmd.postal_code,
            cmd.country,
            cmd.notes,
            cmd.requires_cash_collection,
            cmd.cash_amount,
        )?;

        self.deps.shipment_repo.save(&shipment).await?;
        self.record_event(
            &shipment,
            ShipmentStatus::Pending,
            TrackingEventSource::System,
            Some(actor),
            Some("Shipment created".to_string()),
            None,
        )
        .await?;

        Ok(ShipmentResponse::from(shipment))
    }

    async fn record_event(
        &self,
        shipment: &Shipment,
        status: ShipmentStatus,
        source: TrackingEventSource,
        actor: Option<UserId>,
        notes: Option<String>,
        raw_payload: Option<String>,
    ) -> Result<(), ShippingError> {
        let event = ShipmentTrackingEvent::record(
            shipment.id(),
            status,
            source,
            actor,
            notes,
            None,
            None,
            raw_payload,
        );
        self.deps.event_repo.save(&event).await
    }
}

// =============================================================================
// MarkReadyForPickupUseCase  (StorePickup)
// =============================================================================

pub struct MarkReadyForPickupUseCase {
    deps: Arc<ShipmentDeps>,
}

impl MarkReadyForPickupUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        shipment_id: Uuid,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(shipment_id))?;

        shipment.mark_ready_for_pickup(Duration::hours(DEFAULT_PICKUP_WINDOW_HOURS))?;
        self.deps.shipment_repo.update(&shipment).await?;
        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(format!(
                "Ready for pickup. Code: {}",
                shipment.pickup_code().unwrap_or("?")
            )),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// ConfirmPickupUseCase  (StorePickup)
// =============================================================================

pub struct ConfirmPickupUseCase {
    deps: Arc<ShipmentDeps>,
}

impl ConfirmPickupUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: ConfirmPickupCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        // Constant-time-ish comparison would be nicer, but a 6-digit code over
        // an authenticated channel is fine.
        match shipment.pickup_code() {
            Some(c) if c == cmd.pickup_code => {}
            _ => return Err(ShippingError::InvalidPickupCode),
        }

        shipment.mark_picked_up(cmd.picked_up_by_name.clone())?;
        self.deps.shipment_repo.update(&shipment).await?;

        // COD bridge — pickup at the counter typically already paid in cash,
        // but if the sale carries a pending COD transaction, confirm it.
        confirm_cod_if_needed(&self.deps, &shipment, actor).await?;

        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(format!("Picked up by {}", cmd.picked_up_by_name)),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// AssignDriverUseCase  (OwnDelivery)
// =============================================================================

pub struct AssignDriverUseCase {
    deps: Arc<ShipmentDeps>,
}

impl AssignDriverUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: AssignDriverCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        let driver_id = DriverId::from_uuid(cmd.driver_id);
        let mut driver = self
            .deps
            .driver_repo
            .find_by_id(driver_id)
            .await?
            .ok_or(ShippingError::DriverNotFound(cmd.driver_id))?;

        if !driver.is_active() {
            return Err(ShippingError::DriverNotActive);
        }
        if matches!(driver.current_status(), DriverStatus::Busy) {
            return Err(ShippingError::DriverBusy);
        }

        shipment.assign_driver(driver_id)?;
        driver.set_status(DriverStatus::Busy);

        self.deps.shipment_repo.update(&shipment).await?;
        self.deps.driver_repo.update(&driver).await?;
        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(format!("Assigned to driver {}", driver.name())),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// RescheduleShipmentUseCase  (Failed → Assigned with new driver)
// =============================================================================

pub struct RescheduleShipmentUseCase {
    deps: Arc<ShipmentDeps>,
}

impl RescheduleShipmentUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: RescheduleShipmentCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        let driver_id = DriverId::from_uuid(cmd.new_driver_id);
        let mut driver = self
            .deps
            .driver_repo
            .find_by_id(driver_id)
            .await?
            .ok_or(ShippingError::DriverNotFound(cmd.new_driver_id))?;

        if !driver.is_active() {
            return Err(ShippingError::DriverNotActive);
        }

        shipment.reschedule_with_driver(driver_id)?;
        driver.set_status(DriverStatus::Busy);

        self.deps.shipment_repo.update(&shipment).await?;
        self.deps.driver_repo.update(&driver).await?;
        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(format!(
                "Rescheduled (attempt {}) with driver {}",
                shipment.attempt_count(),
                driver.name()
            )),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// DispatchProviderUseCase  (ExternalDelivery)
// =============================================================================

pub struct DispatchProviderUseCase {
    deps: Arc<ShipmentDeps>,
}

impl DispatchProviderUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: DispatchProviderCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        if !matches!(shipment.method_type(), ShippingMethodType::ExternalDelivery) {
            return Err(ShippingError::ProviderAssignmentNotAllowed);
        }

        let provider_id = match cmd.delivery_provider_id {
            Some(id) => DeliveryProviderId::from_uuid(id),
            None => self
                .deps
                .provider_repo
                .find_default(shipment.store_id())
                .await?
                .map(|p| p.id())
                .ok_or(ShippingError::DeliveryProviderNotFound(Uuid::nil()))?,
        };
        let provider = self
            .deps
            .provider_repo
            .find_by_id(provider_id)
            .await?
            .ok_or_else(|| ShippingError::DeliveryProviderNotFound(provider_id.into_uuid()))?;
        if !provider.is_active() {
            return Err(ShippingError::ProviderNotActive(provider_id.into_uuid()));
        }

        let adapter = self
            .deps
            .provider_registry
            .for_type(provider.provider_type());

        // Try the adapter. Manual adapter always succeeds; stubs error out
        // with `ProviderError(NOT_WIRED)`.
        let dispatch_result = adapter
            .dispatch(DispatchRequest {
                shipment_idempotency_key: shipment.id().into_uuid().to_string(),
                recipient_name: shipment.recipient_name().to_string(),
                recipient_phone: shipment.recipient_phone().map(str::to_string),
                address_line1: shipment.address_line1().to_string(),
                address_line2: shipment.address_line2().map(str::to_string),
                city: shipment.city().to_string(),
                state: shipment.state().to_string(),
                postal_code: shipment.postal_code().map(str::to_string),
                country: shipment.country().to_string(),
                weight_kg: shipment.weight_kg(),
                cash_to_collect: if shipment.requires_cash_collection() {
                    shipment.cash_amount()
                } else {
                    None
                },
                notes: shipment.notes().map(str::to_string),
            })
            .await?;

        // Manual adapters: caller-provided tracking takes precedence.
        let (tracking, carrier) = if adapter.is_manual() {
            (
                cmd.manual_tracking_number
                    .or(Some(dispatch_result.provider_tracking_id)),
                cmd.manual_carrier_name
                    .or(dispatch_result.carrier_name)
                    .or_else(|| Some(provider.name().to_string())),
            )
        } else {
            (
                Some(dispatch_result.provider_tracking_id),
                dispatch_result.carrier_name,
            )
        };

        shipment.dispatch_to_provider(
            provider.id(),
            tracking,
            carrier,
            cmd.estimated_delivery
                .or(dispatch_result.estimated_delivery),
        )?;

        self.deps.shipment_repo.update(&shipment).await?;
        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::Provider,
            Some(actor),
            Some(format!("Dispatched via {}", provider.name())),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// UpdateShipmentStatusUseCase   (generic InTransit / OutForDelivery)
// =============================================================================

pub struct UpdateShipmentStatusUseCase {
    deps: Arc<ShipmentDeps>,
}

impl UpdateShipmentStatusUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: UpdateShipmentStatusCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        let status = ShipmentStatus::from_str(&cmd.status)?;
        shipment.transition_to(status)?;
        self.deps.shipment_repo.update(&shipment).await?;

        let event = ShipmentTrackingEvent::record(
            shipment.id(),
            status,
            TrackingEventSource::System,
            Some(actor),
            cmd.notes,
            cmd.location_lat,
            cmd.location_lng,
            None,
        );
        self.deps.event_repo.save(&event).await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// MarkDeliveredUseCase  (with COD bridge)
// =============================================================================

pub struct MarkDeliveredUseCase {
    deps: Arc<ShipmentDeps>,
}

impl MarkDeliveredUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: MarkDeliveredCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        shipment.mark_delivered()?;
        self.deps.shipment_repo.update(&shipment).await?;

        // Free the driver if it was OwnDelivery.
        if let Some(driver_id) = shipment.driver_id()
            && let Some(mut driver) = self.deps.driver_repo.find_by_id(driver_id).await?
        {
            driver.set_status(DriverStatus::Available);
            self.deps.driver_repo.update(&driver).await?;
        }

        // COD bridge: confirm the pending payment transaction for this sale.
        confirm_cod_if_needed(&self.deps, &shipment, actor).await?;

        let event = ShipmentTrackingEvent::record(
            shipment.id(),
            ShipmentStatus::Delivered,
            TrackingEventSource::System,
            Some(actor),
            cmd.notes,
            cmd.location_lat,
            cmd.location_lng,
            None,
        );
        self.deps.event_repo.save(&event).await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// MarkFailedUseCase
// =============================================================================

pub struct MarkFailedUseCase {
    deps: Arc<ShipmentDeps>,
}

impl MarkFailedUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: MarkFailedCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        shipment.mark_failed(cmd.reason.clone())?;
        self.deps.shipment_repo.update(&shipment).await?;

        // Free driver so they can be reassigned.
        if let Some(driver_id) = shipment.driver_id()
            && let Some(mut driver) = self.deps.driver_repo.find_by_id(driver_id).await?
        {
            driver.set_status(DriverStatus::Available);
            self.deps.driver_repo.update(&driver).await?;
        }

        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(cmd.reason),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// CancelShipmentUseCase
// =============================================================================

pub struct CancelShipmentUseCase {
    deps: Arc<ShipmentDeps>,
}

impl CancelShipmentUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: CancelShipmentCommand,
        actor: UserId,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;

        shipment.cancel(cmd.reason.clone())?;
        self.deps.shipment_repo.update(&shipment).await?;

        if let Some(driver_id) = shipment.driver_id()
            && let Some(mut driver) = self.deps.driver_repo.find_by_id(driver_id).await?
        {
            driver.set_status(DriverStatus::Available);
            self.deps.driver_repo.update(&driver).await?;
        }

        record_status_event(
            &self.deps,
            &shipment,
            TrackingEventSource::System,
            Some(actor),
            Some(cmd.reason),
        )
        .await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// UpdateTrackingUseCase
// =============================================================================

pub struct UpdateTrackingUseCase {
    deps: Arc<ShipmentDeps>,
}

impl UpdateTrackingUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: UpdateTrackingCommand,
    ) -> Result<ShipmentResponse, ShippingError> {
        let id = ShipmentId::from_uuid(cmd.shipment_id);
        let mut shipment = self
            .deps
            .shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShipmentNotFound(cmd.shipment_id))?;
        shipment.set_tracking(cmd.tracking_number, cmd.carrier_name);
        if let Some(eta) = cmd.estimated_delivery {
            shipment.set_estimated_delivery(Some(eta));
        }
        self.deps.shipment_repo.update(&shipment).await?;
        Ok(ShipmentResponse::from(shipment))
    }
}

// =============================================================================
// Read use cases
// =============================================================================

pub struct GetShipmentUseCase {
    deps: Arc<ShipmentDeps>,
}

impl GetShipmentUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }
    pub async fn execute(&self, id: Uuid) -> Result<ShipmentResponse, ShippingError> {
        let shipment = self
            .deps
            .shipment_repo
            .find_by_id(ShipmentId::from_uuid(id))
            .await?
            .ok_or(ShippingError::ShipmentNotFound(id))?;
        Ok(ShipmentResponse::from(shipment))
    }
}

pub struct ListShipmentsUseCase {
    deps: Arc<ShipmentDeps>,
}

impl ListShipmentsUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }
    pub async fn execute(
        &self,
        query: ListShipmentsQuery,
    ) -> Result<ShipmentListResponse, ShippingError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(50).clamp(1, 200);
        let filter = ShipmentFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            sale_id: query.sale_id.map(SaleId::from_uuid),
            status: query
                .status
                .as_deref()
                .map(ShipmentStatus::from_str)
                .transpose()?,
            method_type: query
                .method_type
                .as_deref()
                .map(ShippingMethodType::from_str)
                .transpose()?,
            driver_id: query.driver_id.map(DriverId::from_uuid),
            date_from: query.date_from,
            date_to: query.date_to,
            search: query.search,
        };
        let (rows, total) = self
            .deps
            .shipment_repo
            .find_paginated(filter, page, page_size)
            .await?;
        Ok(ShipmentListResponse {
            items: rows.into_iter().map(ShipmentResponse::from).collect(),
            total,
            page,
            page_size,
        })
    }
}

// =============================================================================
// ExpireStalePickupsUseCase  (background job)
// =============================================================================

pub struct ExpireStalePickupsUseCase {
    deps: Arc<ShipmentDeps>,
}

impl ExpireStalePickupsUseCase {
    pub fn new(deps: Arc<ShipmentDeps>) -> Self {
        Self { deps }
    }

    pub async fn execute(&self) -> Result<i64, ShippingError> {
        let stale = self.deps.shipment_repo.find_expired_pickups().await?;
        let mut count = 0;
        for mut shipment in stale {
            if shipment.mark_expired().is_err() {
                continue;
            }
            self.deps.shipment_repo.update(&shipment).await?;
            let event = ShipmentTrackingEvent::record(
                shipment.id(),
                ShipmentStatus::Expired,
                TrackingEventSource::System,
                None,
                Some("Pickup window passed".to_string()),
                None,
                None,
                None,
            );
            self.deps.event_repo.save(&event).await?;
            count += 1;
        }
        Ok(count)
    }
}

// =============================================================================
// Helpers
// =============================================================================

async fn record_status_event(
    deps: &Arc<ShipmentDeps>,
    shipment: &Shipment,
    source: TrackingEventSource,
    actor: Option<UserId>,
    notes: Option<String>,
) -> Result<(), ShippingError> {
    let event = ShipmentTrackingEvent::record(
        shipment.id(),
        shipment.status(),
        source,
        actor,
        notes,
        None,
        None,
        None,
    );
    deps.event_repo.save(&event).await
}

/// COD bridge — when a shipment carrying cash is settled, find the matching
/// pending charge transaction for the sale and confirm it. Best-effort: a
/// shipment with no matching pending transaction is not considered an error
/// (the sale could have been paid by other means; or be a non-COD pickup).
async fn confirm_cod_if_needed(
    deps: &Arc<ShipmentDeps>,
    shipment: &Shipment,
    actor: UserId,
) -> Result<(), ShippingError> {
    if !shipment.requires_cash_collection() {
        return Ok(());
    }
    let txs = deps
        .transaction_repo
        .find_by_sale_id(shipment.sale_id())
        .await
        .map_err(|e| ShippingError::PaymentConfirmationFailed(e.to_string()))?;
    for mut tx in txs {
        if matches!(tx.status(), payments::TransactionStatus::Pending)
            && matches!(tx.transaction_type(), payments::TransactionType::Charge)
        {
            tx.confirm(actor, shipment.tracking_number().map(str::to_string))
                .map_err(|e| ShippingError::PaymentConfirmationFailed(e.to_string()))?;
            deps.transaction_repo
                .update(&tx)
                .await
                .map_err(|e| ShippingError::PaymentConfirmationFailed(e.to_string()))?;
            // Only one pending charge expected per sale; first match wins.
            break;
        }
    }
    Ok(())
}

// Suppress unused import lint for time reckoning if later removed.
#[allow(dead_code)]
fn _dummy_now() {
    let _ = Utc::now();
}
