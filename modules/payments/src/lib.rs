//! # Payments Module
//!
//! Online payment processing for the POS + eCommerce platform.
//!
//! This module manages:
//! - **Payment Gateways**: per-store gateway configuration (Stripe, PayPal, BAC,
//!   Ficohsa, Manual). Treated as the catalog of "payment methods" the business
//!   can accept; CRUD is restricted to super admins at the API layer.
//! - **Transactions**: charges, refunds, voids backed by an idempotency key.
//! - **Payouts**: settlement records pulled from each gateway.
//!
//! ## Architecture
//!
//! Hexagonal/clean architecture with three layers:
//!
//! - **Domain**: entities, value objects, repository traits
//! - **Application**: use cases and DTOs
//! - **Infrastructure**: PostgreSQL repository implementations + gateway adapters

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API
// =============================================================================

pub use error::PaymentsError;

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

pub use domain::value_objects::PaymentGatewayId;
pub use domain::value_objects::PayoutId;
pub use domain::value_objects::TransactionId;

pub use domain::value_objects::GatewayConfig;
pub use domain::value_objects::GatewayType;
pub use domain::value_objects::ManualPaymentDetails;
pub use domain::value_objects::ManualPaymentKind;
pub use domain::value_objects::PayoutStatus;
pub use domain::value_objects::TransactionStatus;
pub use domain::value_objects::TransactionType;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------

pub use domain::entities::PaymentGateway;
pub use domain::entities::Payout;
pub use domain::entities::Transaction;

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------

pub use domain::repositories::PaymentGatewayRepository;
pub use domain::repositories::PayoutRepository;
pub use domain::repositories::TransactionFilter;
pub use domain::repositories::TransactionRepository;

// -----------------------------------------------------------------------------
// Infrastructure Layer
// -----------------------------------------------------------------------------

pub use infrastructure::gateways::BacCredomaticAdapter;
pub use infrastructure::gateways::DefaultGatewayAdapterRegistry;
pub use infrastructure::gateways::FicohsaAdapter;
pub use infrastructure::gateways::GatewayAdapter;
pub use infrastructure::gateways::GatewayAdapterRegistry;
pub use infrastructure::gateways::GatewayChargeResult;
pub use infrastructure::gateways::GatewayRefundResult;
pub use infrastructure::gateways::ManualGatewayAdapter;
pub use infrastructure::gateways::PayPalAdapter;
pub use infrastructure::gateways::StripeAdapter;
pub use infrastructure::gateways::WebhookEvent;

pub use infrastructure::persistence::PgPaymentGatewayRepository;
pub use infrastructure::persistence::PgPayoutRepository;
pub use infrastructure::persistence::PgTransactionRepository;

// -----------------------------------------------------------------------------
// Application Layer - DTOs
// -----------------------------------------------------------------------------

pub use application::dtos::ConfigureGatewayCommand;
pub use application::dtos::GatewayResponse;
pub use application::dtos::UpdateGatewayCommand;

pub use application::dtos::BankStatementEntry;
pub use application::dtos::ConfirmTransactionCommand;
pub use application::dtos::ListTransactionsQuery;
pub use application::dtos::ProcessOnlinePaymentCommand;
pub use application::dtos::ProcessRefundCommand;
pub use application::dtos::ReconcilePaymentsCommand;
pub use application::dtos::ReconciliationResponse;
pub use application::dtos::RejectTransactionCommand;
pub use application::dtos::TransactionListResponse;
pub use application::dtos::TransactionResponse;

pub use application::dtos::PayoutListResponse;
pub use application::dtos::PayoutResponse;

pub use application::dtos::WebhookPayload;
pub use application::dtos::WebhookResponse;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------

pub use application::use_cases::ConfigureGatewayUseCase;
pub use application::use_cases::DeleteGatewayUseCase;
pub use application::use_cases::GetGatewayUseCase;
pub use application::use_cases::ListGatewaysUseCase;
pub use application::use_cases::UpdateGatewayUseCase;

pub use application::use_cases::ConfirmTransactionUseCase;
pub use application::use_cases::GetTransactionUseCase;
pub use application::use_cases::HandleWebhookUseCase;
pub use application::use_cases::ListTransactionsUseCase;
pub use application::use_cases::ProcessOnlinePaymentUseCase;
pub use application::use_cases::ProcessRefundUseCase;
pub use application::use_cases::ReconcileManualPaymentsUseCase;
pub use application::use_cases::RejectTransactionUseCase;

pub use application::use_cases::ListPayoutsUseCase;
