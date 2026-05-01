//! Periodic recompute of canonical KPI snapshots.
//!
//! Production deployments should run this on a long interval (e.g. once an
//! hour or every 15 minutes). The default interval here is 30 minutes; the
//! `ANALYTICS_RECOMPUTE_INTERVAL_SECS` env var overrides it.

use std::sync::Arc;
use std::time::Duration;

use analytics::{AnalyticsQueryRepository, KpiSnapshotRepository, RecomputeKpiSnapshotsUseCase};

pub fn spawn(
    queries: Arc<dyn AnalyticsQueryRepository>,
    snapshots: Arc<dyn KpiSnapshotRepository>,
    interval_secs: u64,
) {
    let use_case = RecomputeKpiSnapshotsUseCase::new(queries, snapshots);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // Skip the immediate first tick to avoid running on startup before
        // the rest of the system is ready.
        interval.tick().await;

        loop {
            interval.tick().await;
            // System-wide pass for now: store_id = None across all canonical KPIs.
            match use_case.execute(&[None]).await {
                Ok(written) => {
                    if written > 0 {
                        println!("[analytics-recompute] wrote {} snapshots", written);
                    }
                }
                Err(e) => {
                    eprintln!("[analytics-recompute] error: {}", e);
                }
            }
        }
    });
}
