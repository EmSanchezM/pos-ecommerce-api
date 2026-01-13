use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::health::{application::HealthCheckUseCase, domain::HealthStatus};

/// Handler de Axum para el endpoint de health check
pub async fn health_check_handler(
    State(use_case): State<Arc<HealthCheckUseCase>>,
) -> (StatusCode, Json<HealthStatus>) {
    let status = use_case.execute();
    let http_status = if status.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (http_status, Json(status))
}

/// Versión simplificada sin estado compartido
pub async fn health_check_simple() -> (StatusCode, Json<HealthStatus>) {
    let status = HealthStatus::healthy("api-gateway", env!("CARGO_PKG_VERSION"));
    (StatusCode::OK, Json(status))
}
