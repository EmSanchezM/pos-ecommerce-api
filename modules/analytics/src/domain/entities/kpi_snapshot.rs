//! KpiSnapshot entity — a precomputed value of a KPI at a point in time, scoped
//! to a (kpi_key, store, time_window) tuple. The recompute job upserts these;
//! dashboards read the latest snapshot for each widget.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::value_objects::{KpiKey, KpiSnapshotId, TimeWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSnapshot {
    id: KpiSnapshotId,
    kpi_key: KpiKey,
    store_id: Option<Uuid>,
    time_window: TimeWindow,
    value: Decimal,
    metadata: JsonValue,
    computed_at: DateTime<Utc>,
}

impl KpiSnapshot {
    pub fn create(
        kpi_key: KpiKey,
        store_id: Option<Uuid>,
        time_window: TimeWindow,
        value: Decimal,
        metadata: JsonValue,
    ) -> Self {
        Self {
            id: KpiSnapshotId::new(),
            kpi_key,
            store_id,
            time_window,
            value,
            metadata,
            computed_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: KpiSnapshotId,
        kpi_key: KpiKey,
        store_id: Option<Uuid>,
        time_window: TimeWindow,
        value: Decimal,
        metadata: JsonValue,
        computed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            kpi_key,
            store_id,
            time_window,
            value,
            metadata,
            computed_at,
        }
    }

    pub fn id(&self) -> KpiSnapshotId {
        self.id
    }
    pub fn kpi_key(&self) -> &KpiKey {
        &self.kpi_key
    }
    pub fn store_id(&self) -> Option<Uuid> {
        self.store_id
    }
    pub fn time_window(&self) -> TimeWindow {
        self.time_window
    }
    pub fn value(&self) -> Decimal {
        self.value
    }
    pub fn metadata(&self) -> &JsonValue {
        &self.metadata
    }
    pub fn computed_at(&self) -> DateTime<Utc> {
        self.computed_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[test]
    fn create_assigns_fresh_id_and_now_timestamp() {
        let before = Utc::now();
        let s = KpiSnapshot::create(
            KpiKey::new(KpiKey::REVENUE_TOTAL),
            None,
            TimeWindow::Today,
            dec!(1234.56),
            json!({}),
        );
        assert_eq!(s.value(), dec!(1234.56));
        assert_eq!(s.time_window(), TimeWindow::Today);
        assert!(s.computed_at() >= before);
    }
}
