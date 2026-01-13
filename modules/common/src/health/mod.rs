pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-exports para facilitar el uso
pub use application::HealthCheckUseCase;
pub use domain::{HealthState, HealthStatus};
pub use infrastructure::{health_check_handler, health_check_simple};
