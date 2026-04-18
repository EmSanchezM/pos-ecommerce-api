// Promotion entity - discount/coupon code management

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SalesError;
use crate::domain::value_objects::{PromotionId, PromotionStatus, PromotionType};
use identity::UserId;

/// Promotion entity representing a discount code or campaign.
///
/// Supports:
/// - Percentage discounts (e.g., 10% off)
/// - Fixed amount discounts (e.g., $5 off)
/// - Buy X Get Y promotions
/// - Minimum purchase requirements
/// - Usage limits (total and per-customer)
/// - Date-based validity
/// - Store-specific or global promotions
/// - Product/category targeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    id: PromotionId,
    code: String,
    name: String,
    description: Option<String>,
    promotion_type: PromotionType,
    status: PromotionStatus,
    discount_value: Decimal,
    buy_quantity: Option<i32>,
    get_quantity: Option<i32>,
    minimum_purchase: Decimal,
    maximum_discount: Option<Decimal>,
    usage_limit: Option<i32>,
    usage_count: i32,
    per_customer_limit: Option<i32>,
    applies_to: String,
    product_ids: Vec<Uuid>,
    category_ids: Vec<Uuid>,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
    store_id: Option<Uuid>,
    created_by_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Promotion {
    /// Creates a new promotion
    pub fn create(
        code: String,
        name: String,
        promotion_type: PromotionType,
        discount_value: Decimal,
        start_date: DateTime<Utc>,
        created_by_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PromotionId::new(),
            code,
            name,
            description: None,
            promotion_type,
            status: PromotionStatus::Active,
            discount_value,
            buy_quantity: None,
            get_quantity: None,
            minimum_purchase: Decimal::ZERO,
            maximum_discount: None,
            usage_limit: None,
            usage_count: 0,
            per_customer_limit: None,
            applies_to: "order".to_string(),
            product_ids: Vec::new(),
            category_ids: Vec::new(),
            start_date,
            end_date: None,
            store_id: None,
            created_by_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Promotion from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PromotionId,
        code: String,
        name: String,
        description: Option<String>,
        promotion_type: PromotionType,
        status: PromotionStatus,
        discount_value: Decimal,
        buy_quantity: Option<i32>,
        get_quantity: Option<i32>,
        minimum_purchase: Decimal,
        maximum_discount: Option<Decimal>,
        usage_limit: Option<i32>,
        usage_count: i32,
        per_customer_limit: Option<i32>,
        applies_to: String,
        product_ids: Vec<Uuid>,
        category_ids: Vec<Uuid>,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
        store_id: Option<Uuid>,
        created_by_id: UserId,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            name,
            description,
            promotion_type,
            status,
            discount_value,
            buy_quantity,
            get_quantity,
            minimum_purchase,
            maximum_discount,
            usage_limit,
            usage_count,
            per_customer_limit,
            applies_to,
            product_ids,
            category_ids,
            start_date,
            end_date,
            store_id,
            created_by_id,
            created_at,
            updated_at,
        }
    }

    /// Validates that this promotion can be applied to a given order total
    pub fn validate_applicable(&self, order_total: Decimal) -> Result<(), SalesError> {
        if !self.status.is_active() {
            return Err(SalesError::PromotionNotActive);
        }

        let now = Utc::now();
        if now < self.start_date {
            return Err(SalesError::PromotionNotActive);
        }
        if let Some(end) = self.end_date
            && now > end
        {
            return Err(SalesError::PromotionNotActive);
        }

        if let Some(limit) = self.usage_limit
            && self.usage_count >= limit
        {
            return Err(SalesError::PromotionUsageLimitExceeded);
        }

        if order_total < self.minimum_purchase {
            return Err(SalesError::MinimumPurchaseNotMet(self.minimum_purchase));
        }

        Ok(())
    }

    /// Calculates the discount amount for a given order total
    pub fn calculate_discount(&self, order_total: Decimal) -> Decimal {
        let raw_discount = match self.promotion_type {
            PromotionType::Percentage => order_total * self.discount_value / Decimal::from(100),
            PromotionType::FixedAmount => self.discount_value,
            PromotionType::BuyXGetY => Decimal::ZERO, // Handled at item level
        };

        // Apply maximum discount cap
        match self.maximum_discount {
            Some(max) if raw_discount > max => max,
            _ => raw_discount,
        }
    }

    /// Increments the usage counter
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
        self.updated_at = Utc::now();
    }

    /// Deactivates the promotion
    pub fn deactivate(&mut self) {
        self.status = PromotionStatus::Inactive;
        self.updated_at = Utc::now();
    }

    /// Activates the promotion
    pub fn activate(&mut self) {
        self.status = PromotionStatus::Active;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================
    pub fn id(&self) -> PromotionId {
        self.id
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn promotion_type(&self) -> PromotionType {
        self.promotion_type
    }
    pub fn status(&self) -> PromotionStatus {
        self.status
    }
    pub fn discount_value(&self) -> Decimal {
        self.discount_value
    }
    pub fn buy_quantity(&self) -> Option<i32> {
        self.buy_quantity
    }
    pub fn get_quantity(&self) -> Option<i32> {
        self.get_quantity
    }
    pub fn minimum_purchase(&self) -> Decimal {
        self.minimum_purchase
    }
    pub fn maximum_discount(&self) -> Option<Decimal> {
        self.maximum_discount
    }
    pub fn usage_limit(&self) -> Option<i32> {
        self.usage_limit
    }
    pub fn usage_count(&self) -> i32 {
        self.usage_count
    }
    pub fn per_customer_limit(&self) -> Option<i32> {
        self.per_customer_limit
    }
    pub fn applies_to(&self) -> &str {
        &self.applies_to
    }
    pub fn product_ids(&self) -> &[Uuid] {
        &self.product_ids
    }
    pub fn category_ids(&self) -> &[Uuid] {
        &self.category_ids
    }
    pub fn start_date(&self) -> DateTime<Utc> {
        self.start_date
    }
    pub fn end_date(&self) -> Option<DateTime<Utc>> {
        self.end_date
    }
    pub fn store_id(&self) -> Option<Uuid> {
        self.store_id
    }
    pub fn created_by_id(&self) -> UserId {
        self.created_by_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }
    pub fn set_discount_value(&mut self, value: Decimal) {
        self.discount_value = value;
        self.updated_at = Utc::now();
    }
    pub fn set_buy_quantity(&mut self, qty: Option<i32>) {
        self.buy_quantity = qty;
        self.updated_at = Utc::now();
    }
    pub fn set_get_quantity(&mut self, qty: Option<i32>) {
        self.get_quantity = qty;
        self.updated_at = Utc::now();
    }
    pub fn set_minimum_purchase(&mut self, amount: Decimal) {
        self.minimum_purchase = amount;
        self.updated_at = Utc::now();
    }
    pub fn set_maximum_discount(&mut self, amount: Option<Decimal>) {
        self.maximum_discount = amount;
        self.updated_at = Utc::now();
    }
    pub fn set_usage_limit(&mut self, limit: Option<i32>) {
        self.usage_limit = limit;
        self.updated_at = Utc::now();
    }
    pub fn set_per_customer_limit(&mut self, limit: Option<i32>) {
        self.per_customer_limit = limit;
        self.updated_at = Utc::now();
    }
    pub fn set_applies_to(&mut self, applies_to: String) {
        self.applies_to = applies_to;
        self.updated_at = Utc::now();
    }
    pub fn set_product_ids(&mut self, ids: Vec<Uuid>) {
        self.product_ids = ids;
        self.updated_at = Utc::now();
    }
    pub fn set_category_ids(&mut self, ids: Vec<Uuid>) {
        self.category_ids = ids;
        self.updated_at = Utc::now();
    }
    pub fn set_start_date(&mut self, date: DateTime<Utc>) {
        self.start_date = date;
        self.updated_at = Utc::now();
    }
    pub fn set_end_date(&mut self, date: Option<DateTime<Utc>>) {
        self.end_date = date;
        self.updated_at = Utc::now();
    }
    pub fn set_store_id(&mut self, store_id: Option<Uuid>) {
        self.store_id = store_id;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_promotion() -> Promotion {
        Promotion::create(
            "SUMMER2026".to_string(),
            "Summer Sale".to_string(),
            PromotionType::Percentage,
            dec!(10),
            Utc::now() - chrono::Duration::hours(1),
            UserId::new(),
        )
    }

    #[test]
    fn test_create_promotion() {
        let promo = create_test_promotion();
        assert_eq!(promo.code(), "SUMMER2026");
        assert_eq!(promo.name(), "Summer Sale");
        assert_eq!(promo.promotion_type(), PromotionType::Percentage);
        assert_eq!(promo.discount_value(), dec!(10));
        assert!(promo.status().is_active());
        assert_eq!(promo.usage_count(), 0);
    }

    #[test]
    fn test_calculate_discount_percentage() {
        let promo = create_test_promotion();
        let discount = promo.calculate_discount(dec!(100));
        assert_eq!(discount, dec!(10));
    }

    #[test]
    fn test_calculate_discount_fixed() {
        let mut promo = create_test_promotion();
        promo.promotion_type = PromotionType::FixedAmount;
        promo.discount_value = dec!(25);
        let discount = promo.calculate_discount(dec!(100));
        assert_eq!(discount, dec!(25));
    }

    #[test]
    fn test_calculate_discount_with_max_cap() {
        let mut promo = create_test_promotion();
        promo.discount_value = dec!(50); // 50%
        promo.maximum_discount = Some(dec!(20));
        let discount = promo.calculate_discount(dec!(100));
        assert_eq!(discount, dec!(20)); // Capped at 20
    }

    #[test]
    fn test_validate_applicable_success() {
        let promo = create_test_promotion();
        assert!(promo.validate_applicable(dec!(100)).is_ok());
    }

    #[test]
    fn test_validate_applicable_inactive() {
        let mut promo = create_test_promotion();
        promo.deactivate();
        assert!(matches!(
            promo.validate_applicable(dec!(100)),
            Err(SalesError::PromotionNotActive)
        ));
    }

    #[test]
    fn test_validate_applicable_usage_limit() {
        let mut promo = create_test_promotion();
        promo.usage_limit = Some(5);
        promo.usage_count = 5;
        assert!(matches!(
            promo.validate_applicable(dec!(100)),
            Err(SalesError::PromotionUsageLimitExceeded)
        ));
    }

    #[test]
    fn test_validate_applicable_minimum_purchase() {
        let mut promo = create_test_promotion();
        promo.minimum_purchase = dec!(50);
        assert!(matches!(
            promo.validate_applicable(dec!(30)),
            Err(SalesError::MinimumPurchaseNotMet(_))
        ));
    }

    #[test]
    fn test_increment_usage() {
        let mut promo = create_test_promotion();
        assert_eq!(promo.usage_count(), 0);
        promo.increment_usage();
        assert_eq!(promo.usage_count(), 1);
    }

    #[test]
    fn test_deactivate_activate() {
        let mut promo = create_test_promotion();
        assert!(promo.status().is_active());
        promo.deactivate();
        assert!(!promo.status().is_active());
        promo.activate();
        assert!(promo.status().is_active());
    }
}
