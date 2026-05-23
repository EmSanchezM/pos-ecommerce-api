use std::time::Duration;

use sqlx::PgPool;

use crate::config::DatabaseConfig;

pub async fn init_pool(config: &DatabaseConfig) -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .min_connections(config.min_connections)
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
        .connect(&config.url)
        .await
        .expect("Failed to connect to PostgreSQL")
}
