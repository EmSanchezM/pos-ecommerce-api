//! AbcClassification entity — Pareto class assigned to a (variant, store) for a
//! given period. Recomputed monthly by `ClassifyAbcUseCase`.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{AbcClass, AbcClassificationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcClassification {
    id: AbcClassificationId,
    product_variant_id: Uuid,
    store_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
    revenue_share: Decimal,
    abc_class: AbcClass,
    classified_at: DateTime<Utc>,
}

impl AbcClassification {
    pub fn create(
        product_variant_id: Uuid,
        store_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
        revenue_share: Decimal,
        abc_class: AbcClass,
    ) -> Self {
        Self {
            id: AbcClassificationId::new(),
            product_variant_id,
            store_id,
            period_start,
            period_end,
            revenue_share,
            abc_class,
            classified_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AbcClassificationId,
        product_variant_id: Uuid,
        store_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
        revenue_share: Decimal,
        abc_class: AbcClass,
        classified_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            product_variant_id,
            store_id,
            period_start,
            period_end,
            revenue_share,
            abc_class,
            classified_at,
        }
    }

    pub fn id(&self) -> AbcClassificationId {
        self.id
    }
    pub fn product_variant_id(&self) -> Uuid {
        self.product_variant_id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn period_start(&self) -> NaiveDate {
        self.period_start
    }
    pub fn period_end(&self) -> NaiveDate {
        self.period_end
    }
    pub fn revenue_share(&self) -> Decimal {
        self.revenue_share
    }
    pub fn abc_class(&self) -> AbcClass {
        self.abc_class
    }
    pub fn classified_at(&self) -> DateTime<Utc> {
        self.classified_at
    }
}
