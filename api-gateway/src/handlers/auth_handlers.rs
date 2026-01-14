// Authentication HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for user authentication:
// - POST /api/v1/auth/register - Ecommerce user registration
// - POST /api/v1/auth/login - Unified login (email or username)
// - POST /api/v1/auth/refresh - Token refresh

use axum::{extract::State, http::StatusCode, Json};

use identity::{
    LoginCommand, LoginResponse, LoginUseCase, RefreshCommand,
    RefreshTokenUseCase, RegisterEcommerceCommand, RegisterResponse, RegisterUserUseCase,
};

use crate::error::AppError;
use crate::state::AppState;

// =============================================================================
// Register Handler
// =============================================================================

/// Handler for POST /api/v1/auth/register
///
/// Registers a new ecommerce user (customer). The username is auto-generated
/// from the email prefix.
///
/// # Request Body
///
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "securepass123",
///   "first_name": "John",
///   "last_name": "Doe"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: User successfully registered
/// - 400 Bad Request: Validation error (invalid email, password too short, etc.)
/// - 409 Conflict: Email already registered
/// - 500 Internal Server Error: Unexpected error
///
/// - POST /api/v1/auth/register/ecommerce endpoint
/// - HTTP 400 for validation errors
/// - HTTP 500 without exposing internal details
pub async fn register_handler(
    State(state): State<AppState>,
    Json(command): Json<RegisterEcommerceCommand>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    let use_case = RegisterUserUseCase::new(state.user_repo(), state.audit_repo());

    let response = use_case.execute(command).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// Login Handler
// =============================================================================

/// Handler for POST /api/v1/auth/login
///
/// Authenticates a user with email or username and password.
/// Returns JWT access and refresh tokens on success.
///
/// # Request Body
///
/// ```json
/// {
///   "identifier": "user@example.com",
///   "password": "securepass123"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Login successful, returns tokens
/// - 400 Bad Request: Malformed request body
/// - 401 Unauthorized: Invalid credentials or account disabled
/// - 500 Internal Server Error: Unexpected error
///
/// - POST /api/v1/auth/login endpoint
/// - HTTP 400 for malformed requests
/// - HTTP 401 for authentication failures
/// - HTTP 500 without exposing internal details
pub async fn login_handler(
    State(state): State<AppState>,
    Json(command): Json<LoginCommand>,
) -> Result<Json<LoginResponse>, AppError> {
    let use_case = LoginUseCase::new(state.user_repo(), state.token_service());

    let response = use_case.execute(command).await?;

    Ok(Json(response))
}

// =============================================================================
// Refresh Handler
// =============================================================================

/// Handler for POST /api/v1/auth/refresh
///
/// Refreshes an access token using a valid refresh token.
/// Returns a new access token (refresh token remains the same).
///
/// # Request Body
///
/// ```json
/// {
///   "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Token refreshed successfully
/// - 400 Bad Request: Malformed request body
/// - 401 Unauthorized: Invalid or expired refresh token
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - POST /api/v1/auth/refresh endpoint
/// - HTTP 400 for malformed requests
/// - HTTP 401 for invalid tokens
/// - HTTP 500 without exposing internal details
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(command): Json<RefreshCommand>,
) -> Result<Json<LoginResponse>, AppError> {
    let use_case = RefreshTokenUseCase::new(state.user_repo(), state.token_service());

    let response = use_case.execute(command.refresh_token).await?;

    Ok(Json(response))
}
