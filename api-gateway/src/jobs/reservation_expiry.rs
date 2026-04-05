use std::sync::Arc;
use std::time::Duration;

use inventory::{ExpireReservationsUseCase, PgInventoryStockRepository, PgReservationRepository};

/// Spawns a background task that periodically expires pending reservations.
pub fn spawn(
    reservation_repo: Arc<PgReservationRepository>,
    stock_repo: Arc<PgInventoryStockRepository>,
    interval_secs: u64,
) {
    let use_case = ExpireReservationsUseCase::new(reservation_repo, stock_repo);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // First tick completes immediately; skip it to avoid running on startup
        interval.tick().await;

        loop {
            interval.tick().await;
            match use_case.execute().await {
                Ok(result) => {
                    if result.expired_count > 0 || result.failed_count > 0 {
                        println!(
                            "[reservation-expiry] expired={}, failed={}",
                            result.expired_count, result.failed_count
                        );
                    }
                }
                Err(e) => {
                    eprintln!("[reservation-expiry] error: {}", e);
                }
            }
        }
    });
}
