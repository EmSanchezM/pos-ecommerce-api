// Tipos compartidos: Money, Errors, validators, etc.

pub mod health;

// Re-exports principales
pub use health::{HealthCheckUseCase, HealthState, HealthStatus};
