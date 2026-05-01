// Webhook handler - intentionally NOT behind the auth middleware. Each
// gateway is expected to validate its own signature inside `verify_webhook`.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};

use crate::error::AppError;
use crate::state::AppState;
use payments::{HandleWebhookUseCase, WebhookPayload, WebhookResponse};

pub async fn handle_webhook_handler(
    State(state): State<AppState>,
    Path(gateway_type): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<WebhookResponse>, Response> {
    // Common signature header conventions across providers.
    let signature = headers
        .get("stripe-signature")
        .or_else(|| headers.get("paypal-transmission-sig"))
        .or_else(|| headers.get("x-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let raw_body = String::from_utf8_lossy(&body).into_owned();

    let payload = WebhookPayload {
        gateway_type,
        raw_body,
        signature,
    };

    let use_case = HandleWebhookUseCase::new(state.gateway_registry(), state.transaction_repo());

    let response = use_case
        .execute(payload)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
