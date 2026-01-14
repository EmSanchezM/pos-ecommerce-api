// Authentication Middleware for JWT validation
//
// This middleware extracts and validates JWT tokens from the Authorization header,
// builds the UserContext with permissions, and injects it into request extensions.
//
// Requirements: 7.1, 7.2, 7.3, 7.6

use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use identity::{
    BuildUserContextUseCase, ErrorResponse, StoreId, TokenService, UserId,
};

use crate::state::AppState;

/// Authentication middleware that validates JWT tokens and builds UserContext.
///
/// This middleware:
/// 1. Extracts the Bearer token from the Authorization header
/// 2. Validates the token using TokenService
/// 3. Extracts user_id from token claims
/// 4. Builds UserContext with permissions using BuildUserContextUseCase
/// 5. Injects UserContext into request extensions for use by handlers
///
/// # Errors
///
/// - Returns 401 Unauthorized if:
///   - Authorization header is missing
///   - Authorization header doesn't use Bearer scheme
///   - Token is invalid or expired
///   - User is not found or inactive
///
/// # Requirements
///
/// - Requirement 7.1: Return 401 if no token present
/// - Requirement 7.2: Return 401 if token invalid or expired
/// - Requirement 7.3: Extract user_id and build UserContext with permissions
/// - Requirement 7.6: Inject UserContext as extractor for handlers
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // 1. Extract token from Authorization header
    let token = match extract_bearer_token(&request) {
        Ok(token) => token,
        Err(response) => return response,
    };

    // 2. Validate token and get claims
    let claims = match state.token_service().validate_access_token(&token) {
        Ok(claims) => claims,
        Err(_) => {
            return unauthorized_response("Invalid or expired token");
        }
    };

    // 3. Extract user_id from claims
    let user_id = UserId::from_uuid(claims.user_id());

    // 4. Get store_id from query params or use a default store context
    // For now, we'll use a default store context. In a real application,
    // this would come from the request (e.g., X-Store-Id header or query param)
    let store_id = extract_store_id(&request);

    // 5. Build UserContext with permissions
    let build_context_use_case = BuildUserContextUseCase::new(state.user_repo());
    let user_context = match build_context_use_case.execute(user_id, store_id).await {
        Ok(ctx) => ctx,
        Err(e) => {
            // Log the error for debugging (in production, use proper logging)
            eprintln!("Failed to build user context: {:?}", e);
            return unauthorized_response("User not found or inactive");
        }
    };

    // 6. Insert UserContext into request extensions
    request.extensions_mut().insert(user_context);

    // 7. Continue to the next handler
    next.run(request).await
}

/// Extracts the Bearer token from the Authorization header.
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
///
/// # Returns
///
/// * `Ok(String)` - The token string if successfully extracted
/// * `Err(Response)` - A 401 Unauthorized response if extraction fails
fn extract_bearer_token(request: &Request<Body>) -> Result<String, Response> {
    // Get Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(header) => {
            // Check for Bearer scheme
            if let Some(token) = header.strip_prefix("Bearer ") {
                if token.is_empty() {
                    Err(unauthorized_response("Token is empty"))
                } else {
                    Ok(token.to_string())
                }
            } else {
                Err(unauthorized_response("Invalid authorization scheme, expected Bearer"))
            }
        }
        None => Err(unauthorized_response("Missing authorization header")),
    }
}

/// Extracts the store_id from the request.
///
/// Looks for store_id in the following order:
/// 1. X-Store-Id header
/// 2. Falls back to a nil UUID (for system-wide operations)
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
///
/// # Returns
///
/// The StoreId extracted from the request or a default value
fn extract_store_id(request: &Request<Body>) -> StoreId {
    // Try to get store_id from X-Store-Id header
    if let Some(store_id_header) = request.headers().get("X-Store-Id") {
        if let Ok(store_id_str) = store_id_header.to_str() {
            if let Ok(uuid) = uuid::Uuid::parse_str(store_id_str) {
                return StoreId::from_uuid(uuid);
            }
        }
    }

    // Default to nil UUID for system-wide operations
    // In a real application, you might want to require a store_id
    // or have a different default behavior
    StoreId::from_uuid(uuid::Uuid::nil())
}

/// Creates a 401 Unauthorized response with a JSON error body.
///
/// # Arguments
///
/// * `message` - The error message to include in the response
///
/// # Returns
///
/// A Response with status 401 and JSON error body
fn unauthorized_response(message: &str) -> Response {
    let error_response = ErrorResponse::new("UNAUTHORIZED", message);
    (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> Request<Body> {
        Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap()
    }

    fn create_request_with_auth(auth_value: &str) -> Request<Body> {
        Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, auth_value)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let request = create_request_with_auth("Bearer valid_token_here");
        let result = extract_bearer_token(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid_token_here");
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let request = create_test_request();
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let request = create_request_with_auth("Basic dXNlcjpwYXNz");
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_empty_token() {
        let request = create_request_with_auth("Bearer ");
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_store_id_from_header() {
        use uuid::{NoContext, Timestamp};
        let store_uuid = uuid::Uuid::new_v7(Timestamp::now(NoContext));
        let request = Request::builder()
            .uri("/test")
            .header("X-Store-Id", store_uuid.to_string())
            .body(Body::empty())
            .unwrap();

        let store_id = extract_store_id(&request);
        assert_eq!(store_id.into_uuid(), store_uuid);
    }

    #[test]
    fn test_extract_store_id_default() {
        let request = create_test_request();
        let store_id = extract_store_id(&request);
        assert_eq!(store_id.into_uuid(), uuid::Uuid::nil());
    }

    #[test]
    fn test_extract_store_id_invalid_uuid() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Store-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let store_id = extract_store_id(&request);
        // Should fall back to nil UUID
        assert_eq!(store_id.into_uuid(), uuid::Uuid::nil());
    }
}
