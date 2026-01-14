// API Gateway Error Handling
//
// This module provides a unified error type for the API Gateway that maps
// domain errors to appropriate HTTP responses.
//
// Requirements: 5.5, 5.6, 5.7, 5.8

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use identity::{AuthError, ErrorResponse, IdentityError};
use pos_core::CoreError;

// =============================================================================
// AppError - Unified API Gateway Error Type
// =============================================================================

/// Unified error type for the API Gateway.
///
/// This struct wraps various domain errors and implements `IntoResponse`
/// to convert them into appropriate HTTP responses with JSON bodies.
///
/// # Error Mapping
///
/// | Domain Error | HTTP Status | Error Code |
/// |-------------|-------------|------------|
/// | InvalidCredentials | 401 | INVALID_CREDENTIALS |
/// | AccountDisabled | 401 | ACCOUNT_DISABLED |
/// | TokenExpired | 401 | TOKEN_EXPIRED |
/// | InvalidToken | 401 | INVALID_TOKEN |
/// | PasswordTooShort | 400 | VALIDATION_ERROR |
/// | InvalidEmailFormat | 400 | VALIDATION_ERROR |
/// | InvalidUsernameFormat | 400 | VALIDATION_ERROR |
/// | InvalidName | 400 | VALIDATION_ERROR |
/// | DuplicateEmail | 409 | DUPLICATE_EMAIL |
/// | DuplicateUsername | 409 | DUPLICATE_USERNAME |
/// | StoreNotFound | 404 | STORE_NOT_FOUND |
/// | Internal | 500 | INTERNAL_ERROR |
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    response: ErrorResponse,
}

impl AppError {
    /// Creates a new AppError with the given status code and error response.
    pub fn new(status: StatusCode, response: ErrorResponse) -> Self {
        Self { status, response }
    }

    /// Returns the HTTP status code for this error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns a reference to the error response.
    pub fn response(&self) -> &ErrorResponse {
        &self.response
    }
}

// =============================================================================
// IntoResponse Implementation
// =============================================================================

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(self.response)).into_response()
    }
}

// =============================================================================
// From<AuthError> Implementation
// =============================================================================

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        let (status, response) = match &err {
            // 401 Unauthorized - Authentication failures
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::invalid_credentials(),
            ),
            AuthError::AccountDisabled => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::account_disabled(),
            ),
            AuthError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::token_expired(),
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::invalid_token(),
            ),

            // 400 Bad Request - Validation errors
            AuthError::PasswordTooShort => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Password too short: minimum 8 characters required"),
            ),
            AuthError::InvalidEmailFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid email format"),
            ),
            AuthError::InvalidUsernameFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid username format"),
            ),
            AuthError::InvalidName(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(msg),
            ),

            // 409 Conflict - Duplicate resources
            AuthError::DuplicateEmail(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::duplicate_email(),
            ),
            AuthError::DuplicateUsername(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::duplicate_username(),
            ),

            // 404 Not Found
            AuthError::StoreNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse::store_not_found(),
            ),

            // 500 Internal Server Error - Don't expose internal details
            AuthError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<IdentityError> Implementation
// =============================================================================

impl From<IdentityError> for AppError {
    fn from(err: IdentityError) -> Self {
        let (status, response) = match &err {
            // 400 Bad Request - Validation errors
            IdentityError::InvalidPermissionFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid permission format"),
            ),
            IdentityError::InvalidEmailFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid email format"),
            ),
            IdentityError::InvalidUsernameFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid username format"),
            ),

            // 409 Conflict - Duplicate resources
            IdentityError::DuplicatePermission(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_PERMISSION", format!("Permission '{}' already exists", name)),
            ),
            IdentityError::DuplicateRole(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_ROLE", format!("Role '{}' already exists", name)),
            ),
            IdentityError::DuplicateEmail(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::duplicate_email(),
            ),
            IdentityError::DuplicateUsername(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::duplicate_username(),
            ),

            // 404 Not Found
            IdentityError::PermissionNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PERMISSION_NOT_FOUND", "Permission not found"),
            ),
            IdentityError::RoleNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("ROLE_NOT_FOUND", "Role not found"),
            ),
            IdentityError::UserNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("USER_NOT_FOUND", "User not found"),
            ),
            IdentityError::StoreNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::store_not_found(),
            ),
            IdentityError::StoreInactive(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::store_not_found(),
            ),
            IdentityError::UserNotInStore(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("USER_NOT_IN_STORE", "User is not a member of this store"),
            ),

            // 403 Forbidden - Protected resources
            IdentityError::ProtectedRoleCannotBeDeleted => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("PROTECTED_ROLE", "Cannot delete system-protected role"),
            ),

            // 401 Unauthorized - Account status
            IdentityError::UserInactive => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::account_disabled(),
            ),

            // 500 Internal Server Error - Database and other errors
            IdentityError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            IdentityError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<CoreError> Implementation
// =============================================================================

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        let (status, response) = match &err {
            // 404 Not Found
            CoreError::StoreNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("STORE_NOT_FOUND", format!("Store not found: {}", id)),
            ),
            CoreError::TerminalNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("TERMINAL_NOT_FOUND", format!("Terminal not found: {}", id)),
            ),

            // 400 Bad Request - Business rule violations
            CoreError::StoreInactive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("STORE_INACTIVE", format!("Store is inactive: {}", id)),
            ),
            CoreError::TerminalInactive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("TERMINAL_INACTIVE", format!("Terminal is inactive: {}", id)),
            ),
            CoreError::InvalidTerminalCode => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid terminal code format: must be alphanumeric with hyphens, 3-20 characters"),
            ),
            CoreError::InvalidCaiNumber => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid CAI number format"),
            ),
            CoreError::InvalidCaiRange => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid CAI range: start must be <= end"),
            ),
            CoreError::NoCaiAssigned(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("NO_CAI_ASSIGNED", format!("No CAI assigned to terminal: {}", id)),
            ),
            CoreError::CaiExpired(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("CAI_EXPIRED", format!("CAI has expired for terminal: {}", id)),
            ),
            CoreError::CaiRangeExhausted(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("CAI_RANGE_EXHAUSTED", format!("CAI range exhausted for terminal: {}", id)),
            ),

            // 409 Conflict - Duplicate resources
            CoreError::TerminalCodeExists(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("TERMINAL_CODE_EXISTS", format!("Terminal code already exists: {}", code)),
            ),
            CoreError::CaiRangeOverlap => (
                StatusCode::CONFLICT,
                ErrorResponse::new("CAI_RANGE_OVERLAP", "CAI range overlaps with existing active range"),
            ),

            // 403 Forbidden
            CoreError::Unauthorized => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("FORBIDDEN", "Unauthorized: requires super_admin role"),
            ),

            // 500 Internal Server Error
            CoreError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // =========================================================================
    // AuthError Mapping Tests
    // =========================================================================

    #[test]
    fn test_auth_error_invalid_credentials_maps_to_401() {
        let app_error: AppError = AuthError::InvalidCredentials.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "INVALID_CREDENTIALS");
    }

    #[test]
    fn test_auth_error_account_disabled_maps_to_401() {
        let app_error: AppError = AuthError::AccountDisabled.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "ACCOUNT_DISABLED");
    }

    #[test]
    fn test_auth_error_token_expired_maps_to_401() {
        let app_error: AppError = AuthError::TokenExpired.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "TOKEN_EXPIRED");
    }

    #[test]
    fn test_auth_error_invalid_token_maps_to_401() {
        let app_error: AppError = AuthError::InvalidToken.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "INVALID_TOKEN");
    }

    #[test]
    fn test_auth_error_password_too_short_maps_to_400() {
        let app_error: AppError = AuthError::PasswordTooShort.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_email_format_maps_to_400() {
        let app_error: AppError = AuthError::InvalidEmailFormat.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_username_format_maps_to_400() {
        let app_error: AppError = AuthError::InvalidUsernameFormat.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_name_maps_to_400() {
        let app_error: AppError = AuthError::InvalidName("Name too long".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
        assert_eq!(app_error.response().message, "Name too long");
    }

    #[test]
    fn test_auth_error_duplicate_email_maps_to_409() {
        let app_error: AppError = AuthError::DuplicateEmail("test@example.com".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_EMAIL");
    }

    #[test]
    fn test_auth_error_duplicate_username_maps_to_409() {
        let app_error: AppError = AuthError::DuplicateUsername("john_doe".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_USERNAME");
    }

    #[test]
    fn test_auth_error_store_not_found_maps_to_404() {
        let app_error: AppError = AuthError::StoreNotFound.into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "STORE_NOT_FOUND");
    }

    #[test]
    fn test_auth_error_internal_maps_to_500() {
        let app_error: AppError = AuthError::Internal("Database error".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_error.response().error_code, "INTERNAL_ERROR");
        // Internal details should not be exposed
        assert_eq!(app_error.response().message, "Internal error");
    }

    // =========================================================================
    // IdentityError Mapping Tests
    // =========================================================================

    #[test]
    fn test_identity_error_user_not_found_maps_to_404() {
        let app_error: AppError = IdentityError::UserNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "USER_NOT_FOUND");
    }

    #[test]
    fn test_identity_error_protected_role_maps_to_403() {
        let app_error: AppError = IdentityError::ProtectedRoleCannotBeDeleted.into();
        assert_eq!(app_error.status(), StatusCode::FORBIDDEN);
        assert_eq!(app_error.response().error_code, "PROTECTED_ROLE");
    }

    #[test]
    fn test_identity_error_user_inactive_maps_to_401() {
        let app_error: AppError = IdentityError::UserInactive.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "ACCOUNT_DISABLED");
    }
}
