//! ReorderPolicy entity — per-(variant, store) configuration for automatic
//! replenishment. `version` enables optimistic locking on updates so two
//! concurrent edits don't silently overwrite each other.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::value_objects::ReorderPolicyId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderPolicy {
    id: ReorderPolicyId,
    product_variant_id: Uuid,
    store_id: Uuid,
    min_qty: Decimal,
    max_qty: Decimal,
    lead_time_days: i32,
    safety_stock_qty: Decimal,
    review_cycle_days: i32,
    preferred_vendor_id: Option<Uuid>,
    is_active: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ReorderPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        product_variant_id: Uuid,
        store_id: Uuid,
        min_qty: Decimal,
        max_qty: Decimal,
        lead_time_days: i32,
        safety_stock_qty: Decimal,
        review_cycle_days: i32,
        preferred_vendor_id: Option<Uuid>,
    ) -> Result<Self, DemandPlanningError> {
        if min_qty < Decimal::ZERO || max_qty < Decimal::ZERO || safety_stock_qty < Decimal::ZERO {
            return Err(DemandPlanningError::NegativeQuantity);
        }
        if max_qty < min_qty {
            return Err(DemandPlanningError::InvalidPolicyRange);
        }
        if lead_time_days <= 0 || review_cycle_days <= 0 {
            return Err(DemandPlanningError::InvalidPolicyDays);
        }
        let now = Utc::now();
        Ok(Self {
            id: ReorderPolicyId::new(),
            product_variant_id,
            store_id,
            min_qty,
            max_qty,
            lead_time_days,
            safety_stock_qty,
            review_cycle_days,
            preferred_vendor_id,
            is_active: true,
            version: 0,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ReorderPolicyId,
        product_variant_id: Uuid,
        store_id: Uuid,
        min_qty: Decimal,
        max_qty: Decimal,
        lead_time_days: i32,
        safety_stock_qty: Decimal,
        review_cycle_days: i32,
        preferred_vendor_id: Option<Uuid>,
        is_active: bool,
        version: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            product_variant_id,
            store_id,
            min_qty,
            max_qty,
            lead_time_days,
            safety_stock_qty,
            review_cycle_days,
            preferred_vendor_id,
            is_active,
            version,
            created_at,
            updated_at,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        min_qty: Decimal,
        max_qty: Decimal,
        lead_time_days: i32,
        safety_stock_qty: Decimal,
        review_cycle_days: i32,
        preferred_vendor_id: Option<Uuid>,
    ) -> Result<(), DemandPlanningError> {
        if min_qty < Decimal::ZERO || max_qty < Decimal::ZERO || safety_stock_qty < Decimal::ZERO {
            return Err(DemandPlanningError::NegativeQuantity);
        }
        if max_qty < min_qty {
            return Err(DemandPlanningError::InvalidPolicyRange);
        }
        if lead_time_days <= 0 || review_cycle_days <= 0 {
            return Err(DemandPlanningError::InvalidPolicyDays);
        }
        self.min_qty = min_qty;
        self.max_qty = max_qty;
        self.lead_time_days = lead_time_days;
        self.safety_stock_qty = safety_stock_qty;
        self.review_cycle_days = review_cycle_days;
        self.preferred_vendor_id = preferred_vendor_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn id(&self) -> ReorderPolicyId {
        self.id
    }
    pub fn product_variant_id(&self) -> Uuid {
        self.product_variant_id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn min_qty(&self) -> Decimal {
        self.min_qty
    }
    pub fn max_qty(&self) -> Decimal {
        self.max_qty
    }
    pub fn lead_time_days(&self) -> i32 {
        self.lead_time_days
    }
    pub fn safety_stock_qty(&self) -> Decimal {
        self.safety_stock_qty
    }
    pub fn review_cycle_days(&self) -> i32 {
        self.review_cycle_days
    }
    pub fn preferred_vendor_id(&self) -> Option<Uuid> {
        self.preferred_vendor_id
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn fresh_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    fn ids() -> (Uuid, Uuid) {
        (fresh_uuid(), fresh_uuid())
    }

    #[test]
    fn create_rejects_max_below_min() {
        let (v, s) = ids();
        let err = ReorderPolicy::create(v, s, dec!(50), dec!(10), 7, dec!(0), 7, None).unwrap_err();
        assert!(matches!(err, DemandPlanningError::InvalidPolicyRange));
    }

    #[test]
    fn create_rejects_zero_lead_time() {
        let (v, s) = ids();
        let err = ReorderPolicy::create(v, s, dec!(10), dec!(50), 0, dec!(0), 7, None).unwrap_err();
        assert!(matches!(err, DemandPlanningError::InvalidPolicyDays));
    }

    #[test]
    fn create_rejects_negative_safety_stock() {
        let (v, s) = ids();
        let err =
            ReorderPolicy::create(v, s, dec!(10), dec!(50), 7, dec!(-1), 7, None).unwrap_err();
        assert!(matches!(err, DemandPlanningError::NegativeQuantity));
    }

    #[test]
    fn create_starts_active_at_version_zero() {
        let (v, s) = ids();
        let p = ReorderPolicy::create(v, s, dec!(10), dec!(50), 7, dec!(2), 7, None).unwrap();
        assert!(p.is_active());
        assert_eq!(p.version(), 0);
    }
}
