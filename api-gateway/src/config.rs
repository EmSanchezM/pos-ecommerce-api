use std::env;
use std::str::FromStr;

use common::TrustedProxies;

/// Comma-separated IPs / CIDR blocks of reverse proxies whose
/// `X-Forwarded-For` may be believed when rate limiting.
///
/// Unset means "trust nothing", which keys every request on the address that
/// actually connected. Behind Caddy this MUST list the proxy, or every tenant
/// shares a single auth-rate-limit bucket.
const TRUSTED_PROXY_IPS: &str = "TRUSTED_PROXY_IPS";

pub struct AppConfig {
    pub database: DatabaseConfig,
    pub jwt_secret: String,
    /// Shared secret authenticating service-to-service calls to `/internal/*`
    /// (e.g. backoffice-api requesting an impersonation token). Both binaries
    /// must be configured with the same value.
    pub internal_service_secret: String,
    pub cors_allowed_origins: Option<String>,
    /// Reverse proxies whose forwarded headers are trusted for rate limiting.
    pub trusted_proxies: TrustedProxies,
    pub image_storage: ImageStorageConfig,
    pub jobs: JobsConfig,
}

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

pub struct ImageStorageConfig {
    pub public_url: String,
    pub root: String,
}

pub struct JobsConfig {
    pub reservation_expiry_interval: u64,
    pub cart_cleanup_interval: u64,
    pub event_dispatch_interval: u64,
    pub event_dispatch_batch_size: i64,
    pub notification_retry_interval: u64,
    pub notification_retry_batch_size: i64,
    pub analytics_recompute_interval: u64,
    pub demand_planning_interval: u64,
    pub subscription_billing_interval: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                max_connections: env_or("DB_MAX_CONNECTIONS", 50),
                min_connections: env_or("DB_MIN_CONNECTIONS", 5),
                acquire_timeout_secs: env_or("DB_ACQUIRE_TIMEOUT_SECS", 5),
                idle_timeout_secs: env_or("DB_IDLE_TIMEOUT_SECS", 300),
                max_lifetime_secs: env_or("DB_MAX_LIFETIME_SECS", 1800),
            },
            internal_service_secret: env::var("INTERNAL_SERVICE_SECRET")
                .expect("INTERNAL_SERVICE_SECRET must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET environment variable must be set"),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .ok()
                .filter(|s| !s.is_empty()),
            trusted_proxies: TrustedProxies::from_env(TRUSTED_PROXY_IPS),
            image_storage: ImageStorageConfig {
                public_url: env::var("IMAGE_STORAGE_PUBLIC_URL")
                    .unwrap_or_else(|_| "/uploads".to_string()),
                root: env::var("IMAGE_STORAGE_ROOT").unwrap_or_else(|_| "./uploads".to_string()),
            },
            jobs: JobsConfig {
                reservation_expiry_interval: env_or("RESERVATION_EXPIRY_INTERVAL_SECS", 300),
                cart_cleanup_interval: env_or("CART_CLEANUP_INTERVAL_SECS", 900),
                event_dispatch_interval: env_or("EVENT_DISPATCH_INTERVAL_SECS", 5),
                event_dispatch_batch_size: env_or("EVENT_DISPATCH_BATCH_SIZE", 100),
                notification_retry_interval: env_or("NOTIFICATION_RETRY_INTERVAL_SECS", 60),
                notification_retry_batch_size: env_or("NOTIFICATION_RETRY_BATCH_SIZE", 50),
                analytics_recompute_interval: env_or("ANALYTICS_RECOMPUTE_INTERVAL_SECS", 1800),
                demand_planning_interval: env_or("DEMAND_PLANNING_RECOMPUTE_INTERVAL_SECS", 86_400),
                subscription_billing_interval: env_or("SUBSCRIPTION_BILLING_INTERVAL_SECS", 3600),
            },
        }
    }
}

fn env_or<T: FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
