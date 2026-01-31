//! Value objects for the sales domain.
//!
//! This module contains all value objects used in the sales module,
//! including IDs, status enums, and other immutable typed values.

// ID value objects
mod cart_id;
mod cart_item_id;
mod credit_note_id;
mod credit_note_item_id;
mod customer_id;
mod payment_id;
mod sale_id;
mod sale_item_id;
mod shift_id;

// Enum value objects
mod credit_note_status;
mod customer_type;
mod discount_type;
mod order_status;
mod payment_method;
mod payment_status;
mod return_reason;
mod return_type;
mod sale_status;
mod sale_type;
mod shift_status;

// Re-exports - IDs
pub use cart_id::CartId;
pub use cart_item_id::CartItemId;
pub use credit_note_id::CreditNoteId;
pub use credit_note_item_id::CreditNoteItemId;
pub use customer_id::CustomerId;
pub use payment_id::PaymentId;
pub use sale_id::SaleId;
pub use sale_item_id::SaleItemId;
pub use shift_id::ShiftId;

// Re-exports - Enums
pub use credit_note_status::CreditNoteStatus;
pub use customer_type::CustomerType;
pub use discount_type::DiscountType;
pub use order_status::OrderStatus;
pub use payment_method::PaymentMethod;
pub use payment_status::PaymentStatus;
pub use return_reason::ReturnReason;
pub use return_type::ReturnType;
pub use sale_status::SaleStatus;
pub use sale_type::SaleType;
pub use shift_status::ShiftStatus;
