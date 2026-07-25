// Tipos compartidos: Money, Errors, validators, etc.

pub mod auth;
pub mod health;
pub mod rate_limit;

pub use auth::{ActorClaim, BackofficeClaims, TokenAudience};
pub use health::{HealthCheckUseCase, HealthState, HealthStatus};
pub use rate_limit::{ClientIpKeyExtractor, TrustedProxies, TrustedProxyParseError};
