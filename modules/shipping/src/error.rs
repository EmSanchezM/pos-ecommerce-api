//! Shipping module error types.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ShippingError {
    // -------------------------------------------------------------------------
    // Configuration errors
    // -------------------------------------------------------------------------
    #[error("Shipping method not found: {0}")]
    ShippingMethodNotFound(Uuid),

    #[error("Shipping zone not found: {0}")]
    ShippingZoneNotFound(Uuid),

    #[error("Shipping rate not found: {0}")]
    ShippingRateNotFound(Uuid),

    #[error("Driver not found: {0}")]
    DriverNotFound(Uuid),

    #[error("Delivery provider not found: {0}")]
    DeliveryProviderNotFound(Uuid),

    #[error("Duplicate shipping method code: {0}")]
    DuplicateMethodCode(String),

    #[error("Duplicate driver phone: {0}")]
    DuplicateDriverPhone(String),

    #[error("Duplicate delivery provider name: {0}")]
    DuplicateProviderName(String),

    // -------------------------------------------------------------------------
    // Calculation errors
    // -------------------------------------------------------------------------
    #[error("No shipping zone matches the destination")]
    NoMatchingZone,

    #[error("No shipping rates available for this zone/method")]
    NoRatesAvailable,

    #[error("Order exceeds maximum weight for this rate")]
    ExceedsMaxWeight,

    #[error("Order below minimum amount for this shipping method")]
    BelowMinimumAmount,

    #[error("Method not available right now (off-hours / closed day)")]
    MethodOutsideAvailability,

    // -------------------------------------------------------------------------
    // Shipment lifecycle
    // -------------------------------------------------------------------------
    #[error("Shipment not found: {0}")]
    ShipmentNotFound(Uuid),

    #[error("Shipment already exists for sale: {0}")]
    ShipmentAlreadyExistsForSale(Uuid),

    #[error("Shipment already delivered")]
    ShipmentAlreadyDelivered,

    #[error("Shipment already cancelled")]
    ShipmentAlreadyCancelled,

    #[error("Invalid shipment status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Driver is not active")]
    DriverNotActive,

    #[error("Driver is busy with another shipment")]
    DriverBusy,

    #[error("This shipment method does not allow driver assignment")]
    DriverAssignmentNotAllowed,

    #[error("This shipment method does not allow external provider")]
    ProviderAssignmentNotAllowed,

    #[error("Pickup code is invalid or expired")]
    InvalidPickupCode,

    #[error("Pickup window has expired")]
    PickupExpired,

    // -------------------------------------------------------------------------
    // External provider errors
    // -------------------------------------------------------------------------
    #[error("Provider not active: {0}")]
    ProviderNotActive(Uuid),

    #[error("Provider does not cover this zone")]
    ProviderZoneNotCovered,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,

    // -------------------------------------------------------------------------
    // Validation errors
    // -------------------------------------------------------------------------
    #[error("Invalid shipping method type")]
    InvalidMethodType,

    #[error("Invalid shipping rate type")]
    InvalidRateType,

    #[error("Invalid shipment status")]
    InvalidShipmentStatus,

    #[error("Invalid driver vehicle type")]
    InvalidVehicleType,

    #[error("Invalid driver status")]
    InvalidDriverStatus,

    #[error("Invalid delivery provider type")]
    InvalidProviderType,

    #[error("Invalid tracking event source")]
    InvalidTrackingSource,

    #[error("Invalid amount: must be non-negative")]
    InvalidAmount,

    // -------------------------------------------------------------------------
    // Cross-module
    // -------------------------------------------------------------------------
    #[error("Sale not found: {0}")]
    SaleNotFound(Uuid),

    #[error("Payment confirmation failed: {0}")]
    PaymentConfirmationFailed(String),

    // -------------------------------------------------------------------------
    // System
    // -------------------------------------------------------------------------
    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not implemented")]
    NotImplemented,
}
