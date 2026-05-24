// Health check handler — GET /health

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Returns HTTP 200 with `{"status": "ok"}` (FR-BIN-4).
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}
