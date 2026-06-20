// Backoffice API configuration
//
// Reads environment variables at startup. Mirrors api-gateway/src/config.rs.

use std::env;
use std::str::FromStr;

pub struct BackofficeConfig {
    pub database: DatabaseConfig,
    pub backoffice_secret: String,
    pub backoffice_issuer: String,
    /// JWT_SECRET — the tenant token signing key.
    ///
    /// Per Decision 2 (sdd/backoffice-api/decisions), backoffice-api reads
    /// JWT_SECRET ONLY to sign impersonation tokens with `aud: Tenant` so
    /// api-gateway can validate them. This is a dev-mode pragmatic choice;
    /// the v2 migration target is an internal mTLS endpoint on api-gateway.
    pub tenant_secret: String,
    pub port: u16,
}

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl BackofficeConfig {
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                max_connections: env_or("DB_MAX_CONNECTIONS", 10),
                min_connections: env_or("DB_MIN_CONNECTIONS", 2),
                acquire_timeout_secs: env_or("DB_ACQUIRE_TIMEOUT_SECS", 5),
                idle_timeout_secs: env_or("DB_IDLE_TIMEOUT_SECS", 300),
                max_lifetime_secs: env_or("DB_MAX_LIFETIME_SECS", 1800),
            },
            backoffice_secret: env::var("JWT_BACKOFFICE_SECRET")
                .expect("JWT_BACKOFFICE_SECRET environment variable must be set"),
            backoffice_issuer: env::var("JWT_BACKOFFICE_ISSUER")
                .unwrap_or_else(|_| "backoffice-api".to_string()),
            // JWT_SECRET is the TENANT signing key — read here ONLY to sign
            // impersonation tokens. See Decision 2 in sdd/backoffice-api/decisions.
            tenant_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET environment variable must be set"),
            port: env_or("BACKOFFICE_PORT", 8001u16),
        }
    }
}

fn env_or<T: FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
