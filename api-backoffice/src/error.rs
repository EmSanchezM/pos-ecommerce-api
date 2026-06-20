// Backoffice API Error Handling
//
// Maps BackofficeIdentityError (and future domain errors) to HTTP responses.
// Mirrors api-gateway/src/error.rs pattern.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use backoffice_identity::BackofficeIdentityError;
use serde::{Deserialize, Serialize};
use tenancy::TenancyError;

// =============================================================================
// ErrorResponse — standard JSON error shape
// =============================================================================

/// Standard JSON error body returned by the backoffice API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message)
    }

    pub fn internal_error() -> Self {
        Self::new("INTERNAL_ERROR", "An internal server error occurred")
    }

    pub fn not_implemented() -> Self {
        Self::new("NOT_IMPLEMENTED", "This endpoint is not yet implemented")
    }
}

// =============================================================================
// AppError — unified error type for the backoffice binary
// =============================================================================

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub body: ErrorResponse,
}

impl AppError {
    pub fn new(status: StatusCode, body: ErrorResponse) -> Self {
        Self { status, body }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            ErrorResponse::unauthorized(message),
        )
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, ErrorResponse::forbidden(message))
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::internal_error(),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

// =============================================================================
// From<BackofficeIdentityError>
// =============================================================================

impl From<BackofficeIdentityError> for AppError {
    fn from(err: BackofficeIdentityError) -> Self {
        let (status, body) = match &err {
            BackofficeIdentityError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("INVALID_CREDENTIALS", "Invalid email or password"),
            ),
            BackofficeIdentityError::UserInactive => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("ACCOUNT_DISABLED", "Backoffice user account is inactive"),
            ),
            BackofficeIdentityError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("INVALID_TOKEN", "Invalid or expired token"),
            ),
            BackofficeIdentityError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("TOKEN_EXPIRED", "Token has expired"),
            ),
            BackofficeIdentityError::InvalidEmailFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VALIDATION_ERROR", "Invalid email format"),
            ),
            BackofficeIdentityError::InvalidPermissionCodeFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VALIDATION_ERROR", "Invalid permission code format"),
            ),
            BackofficeIdentityError::UserNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("USER_NOT_FOUND", "Backoffice user not found"),
            ),
            BackofficeIdentityError::RoleNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("ROLE_NOT_FOUND", "Backoffice role not found"),
            ),
            BackofficeIdentityError::PermissionNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PERMISSION_NOT_FOUND", "Backoffice permission not found"),
            ),
            BackofficeIdentityError::DuplicateEmail(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_EMAIL", "Email already in use"),
            ),
            BackofficeIdentityError::DuplicateRole(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_ROLE", "Role already exists"),
            ),
            BackofficeIdentityError::DuplicatePermission(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_PERMISSION", "Permission already exists"),
            ),
            BackofficeIdentityError::ProtectedRoleCannotBeModified => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("PROTECTED_ROLE", "Cannot modify system-protected role"),
            ),
            BackofficeIdentityError::PasswordHashError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            BackofficeIdentityError::JwtError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            BackofficeIdentityError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),

            // --- Phase 4 variants ---
            BackofficeIdentityError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_INPUT", msg.as_str()),
            ),
            BackofficeIdentityError::OrgNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("ORG_NOT_FOUND", format!("Organization {} not found", id)),
            ),
            BackofficeIdentityError::Tenancy(msg) => {
                tracing::error!("org state transition error: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorResponse::new("STATE_TRANSITION_ERROR", msg.as_str()),
                )
            }
            BackofficeIdentityError::Outbox(msg) => {
                tracing::error!("outbox publish error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::internal_error(),
                )
            }
        };

        AppError::new(status, body)
    }
}

// =============================================================================
// From<TenancyError>
// =============================================================================

impl From<TenancyError> for AppError {
    fn from(err: TenancyError) -> Self {
        let (status, body) = match &err {
            TenancyError::OrganizationNotFound(_)
            | TenancyError::OrganizationNotFoundBySlug(_)
            | TenancyError::PlanNotFound(_)
            | TenancyError::DomainNotFound(_)
            | TenancyError::DomainNotFoundByHostname(_)
            | TenancyError::BrandingNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("NOT_FOUND", err.to_string()),
            ),
            TenancyError::SlugAlreadyTaken(_) | TenancyError::DomainAlreadyTaken(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("CONFLICT", err.to_string()),
            ),
            TenancyError::InvalidStatusTransition { .. }
            | TenancyError::InvalidStatus(_)
            | TenancyError::InvalidTier(_)
            | TenancyError::InvalidTheme(_)
            | TenancyError::InvalidSlug(_)
            | TenancyError::InvalidDomain(_)
            | TenancyError::InvalidColor(_)
            | TenancyError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VALIDATION_ERROR", err.to_string()),
            ),
            TenancyError::Database(_)
            | TenancyError::Serialization(_)
            | TenancyError::Subscriber(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_credentials_maps_to_401() {
        let err: AppError = BackofficeIdentityError::InvalidCredentials.into();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.body.code, "INVALID_CREDENTIALS");
    }

    #[test]
    fn user_inactive_maps_to_401() {
        let err: AppError = BackofficeIdentityError::UserInactive.into();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.body.code, "ACCOUNT_DISABLED");
    }

    #[test]
    fn database_error_maps_to_500() {
        let db_err = sqlx::Error::RowNotFound;
        let err: AppError = BackofficeIdentityError::Database(db_err).into();
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn tenancy_org_not_found_maps_to_404() {
        let err: AppError = TenancyError::OrganizationNotFound(uuid::Uuid::nil()).into();
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.body.code, "NOT_FOUND");
    }

    #[test]
    fn tenancy_database_error_maps_to_500() {
        let err: AppError = TenancyError::Database(sqlx::Error::RowNotFound).into();
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
