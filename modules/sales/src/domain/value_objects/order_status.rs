//! OrderStatus enum - workflow status for E-commerce orders

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for E-commerce orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Waiting for payment
    PendingPayment,
    /// Payment failed
    PaymentFailed,
    /// Payment confirmed
    Paid,
    /// Order in preparation
    Processing,
    /// Order shipped
    Shipped,
    /// Order delivered
    Delivered,
    /// Order cancelled
    Cancelled,
    /// Order returned
    Returned,
}

impl OrderStatus {
    /// Returns all available order statuses
    pub fn all() -> &'static [OrderStatus] {
        &[
            OrderStatus::PendingPayment,
            OrderStatus::PaymentFailed,
            OrderStatus::Paid,
            OrderStatus::Processing,
            OrderStatus::Shipped,
            OrderStatus::Delivered,
            OrderStatus::Cancelled,
            OrderStatus::Returned,
        ]
    }

    /// Returns true if the order is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            OrderStatus::Delivered | OrderStatus::Cancelled | OrderStatus::Returned
        )
    }

    /// Returns true if the order can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderStatus::PendingPayment | OrderStatus::Paid | OrderStatus::Processing
        )
    }

    /// Returns true if the order can be processed
    pub fn can_process(&self) -> bool {
        matches!(self, OrderStatus::Paid)
    }

    /// Returns true if the order can be shipped
    pub fn can_ship(&self) -> bool {
        matches!(self, OrderStatus::Processing)
    }

    /// Returns true if the order can be delivered
    pub fn can_deliver(&self) -> bool {
        matches!(self, OrderStatus::Shipped)
    }

    /// Returns true if a return can be created for this order
    pub fn can_return(&self) -> bool {
        matches!(self, OrderStatus::Delivered)
    }

    /// Returns true if payment can be retried
    pub fn can_retry_payment(&self) -> bool {
        matches!(self, OrderStatus::PaymentFailed)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: OrderStatus) -> bool {
        match (self, new_status) {
            // From PendingPayment
            (OrderStatus::PendingPayment, OrderStatus::Paid) => true,
            (OrderStatus::PendingPayment, OrderStatus::PaymentFailed) => true,
            (OrderStatus::PendingPayment, OrderStatus::Cancelled) => true,
            // From PaymentFailed
            (OrderStatus::PaymentFailed, OrderStatus::Paid) => true,
            (OrderStatus::PaymentFailed, OrderStatus::Cancelled) => true,
            // From Paid
            (OrderStatus::Paid, OrderStatus::Processing) => true,
            (OrderStatus::Paid, OrderStatus::Cancelled) => true,
            // From Processing
            (OrderStatus::Processing, OrderStatus::Shipped) => true,
            (OrderStatus::Processing, OrderStatus::Cancelled) => true,
            // From Shipped
            (OrderStatus::Shipped, OrderStatus::Delivered) => true,
            // From Delivered
            (OrderStatus::Delivered, OrderStatus::Returned) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for OrderStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "pending_payment" | "pendingpayment" | "pending" => Ok(OrderStatus::PendingPayment),
            "payment_failed" | "paymentfailed" | "failed" => Ok(OrderStatus::PaymentFailed),
            "paid" => Ok(OrderStatus::Paid),
            "processing" => Ok(OrderStatus::Processing),
            "shipped" => Ok(OrderStatus::Shipped),
            "delivered" => Ok(OrderStatus::Delivered),
            "cancelled" | "canceled" => Ok(OrderStatus::Cancelled),
            "returned" => Ok(OrderStatus::Returned),
            _ => Err(SalesError::InvalidOrderStatus),
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::PendingPayment => write!(f, "pending_payment"),
            OrderStatus::PaymentFailed => write!(f, "payment_failed"),
            OrderStatus::Paid => write!(f, "paid"),
            OrderStatus::Processing => write!(f, "processing"),
            OrderStatus::Shipped => write!(f, "shipped"),
            OrderStatus::Delivered => write!(f, "delivered"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Returned => write!(f, "returned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            OrderStatus::from_str("pending_payment").unwrap(),
            OrderStatus::PendingPayment
        );
        assert_eq!(
            OrderStatus::from_str("pending").unwrap(),
            OrderStatus::PendingPayment
        );
        assert_eq!(OrderStatus::from_str("paid").unwrap(), OrderStatus::Paid);
        assert_eq!(
            OrderStatus::from_str("cancelled").unwrap(),
            OrderStatus::Cancelled
        );
        assert_eq!(
            OrderStatus::from_str("canceled").unwrap(),
            OrderStatus::Cancelled
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(OrderStatus::PendingPayment.to_string(), "pending_payment");
        assert_eq!(OrderStatus::Delivered.to_string(), "delivered");
    }

    #[test]
    fn test_workflow_states() {
        // PendingPayment
        assert!(OrderStatus::PendingPayment.can_cancel());
        assert!(!OrderStatus::PendingPayment.is_final());

        // Paid
        assert!(OrderStatus::Paid.can_process());
        assert!(OrderStatus::Paid.can_cancel());

        // Processing
        assert!(OrderStatus::Processing.can_ship());
        assert!(OrderStatus::Processing.can_cancel());

        // Shipped
        assert!(OrderStatus::Shipped.can_deliver());
        assert!(!OrderStatus::Shipped.can_cancel());

        // Delivered
        assert!(OrderStatus::Delivered.can_return());
        assert!(OrderStatus::Delivered.is_final());

        // Final states
        assert!(OrderStatus::Cancelled.is_final());
        assert!(OrderStatus::Returned.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From PendingPayment
        assert!(OrderStatus::PendingPayment.can_transition_to(OrderStatus::Paid));
        assert!(OrderStatus::PendingPayment.can_transition_to(OrderStatus::PaymentFailed));
        assert!(OrderStatus::PendingPayment.can_transition_to(OrderStatus::Cancelled));

        // From Paid
        assert!(OrderStatus::Paid.can_transition_to(OrderStatus::Processing));
        assert!(!OrderStatus::Paid.can_transition_to(OrderStatus::Shipped));

        // From Shipped
        assert!(OrderStatus::Shipped.can_transition_to(OrderStatus::Delivered));
        assert!(!OrderStatus::Shipped.can_transition_to(OrderStatus::Cancelled));
    }
}
