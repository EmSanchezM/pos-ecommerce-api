// JsonBody<T> — request extractor that maps axum's JsonRejection to our
// standard ErrorResponse JSON body instead of Axum's plain-text 422.

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use identity::ErrorResponse;
use serde::de::DeserializeOwned;

pub struct JsonBody<T>(pub T);

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(JsonBody(value)),
            Err(rejection) => Err(rejection_to_response(rejection)),
        }
    }
}

fn rejection_to_response(rejection: JsonRejection) -> Response {
    let (status, code, message) = match &rejection {
        JsonRejection::JsonDataError(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_JSON_DATA",
            format!("Invalid request body: {}", e.body_text()),
        ),
        JsonRejection::JsonSyntaxError(e) => (
            StatusCode::BAD_REQUEST,
            "INVALID_JSON_SYNTAX",
            format!("Malformed JSON: {}", e.body_text()),
        ),
        JsonRejection::MissingJsonContentType(_) => (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "MISSING_JSON_CONTENT_TYPE",
            "Expected `Content-Type: application/json`".to_string(),
        ),
        JsonRejection::BytesRejection(_) => (
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST_BODY",
            "Failed to read request body".to_string(),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            rejection.body_text(),
        ),
    };

    (status, Json(ErrorResponse::new(code, message))).into_response()
}
