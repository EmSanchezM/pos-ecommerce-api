// Delivery webhooks — NO auth; the adapter validates the provider signature.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};

use crate::error::AppError;
use crate::state::AppState;
use shipping::{DeliveryWebhookPayload, DeliveryWebhookResponse, HandleDeliveryWebhookUseCase};

pub async fn delivery_webhook_handler(
    State(state): State<AppState>,
    Path(provider_type): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DeliveryWebhookResponse>, Response> {
    // Per-provider signature header conventions.
    let signature = headers
        .get("x-hugo-signature")
        .or_else(|| headers.get("x-pedidosya-signature"))
        .or_else(|| headers.get("x-uber-signature"))
        .or_else(|| headers.get("x-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let raw_body = String::from_utf8_lossy(&body).into_owned();
    let payload = DeliveryWebhookPayload {
        provider_type,
        raw_body,
        signature,
    };

    let uc = HandleDeliveryWebhookUseCase::new(
        state.delivery_registry(),
        state.shipment_repo(),
        state.shipment_event_repo(),
    );
    let resp = uc
        .execute(payload)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
