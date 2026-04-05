use std::sync::Arc;
use std::time::Duration;

use sales::{CartRepository, PgCartRepository};

/// Spawns a background task that periodically deletes expired carts.
pub fn spawn(cart_repo: Arc<PgCartRepository>, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // First tick completes immediately; skip it to avoid running on startup
        interval.tick().await;

        loop {
            interval.tick().await;
            let before = chrono::Utc::now();
            match cart_repo.delete_expired(before).await {
                Ok(count) => {
                    if count > 0 {
                        println!("[cart-cleanup] deleted {} expired carts", count);
                    }
                }
                Err(e) => {
                    eprintln!("[cart-cleanup] error: {}", e);
                }
            }
        }
    });
}
