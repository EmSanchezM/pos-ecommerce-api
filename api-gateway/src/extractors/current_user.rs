// CurrentUser Extractor for Axum
//
// This extractor retrieves the UserContext from request extensions,
// which is injected by the auth middleware.
//

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use identity::{ErrorResponse, UserContext};

/// Extractor for the current authenticated user's context.
///
/// This extractor retrieves the `UserContext` from request extensions,
/// which is injected by the `auth_middleware`. It provides access to
/// the user's ID, store ID, and permissions.
///
/// # Usage
///
/// ```ignore
/// pub async fn my_handler(
///     CurrentUser(ctx): CurrentUser,
///     // other extractors...
/// ) -> Result<Json<Response>, AppError> {
///     // Access user information
///     let user_id = ctx.user_id();
///     let store_id = ctx.store_id();
///     
///     // Check permissions
///     if ctx.has_permission("stores:create") {
///         // ...
///     }
/// }
/// ```
///
/// # Errors
///
/// Returns 401 Unauthorized if the UserContext is not present in the
/// request extensions. This typically means the auth middleware was
/// not applied to the route.
///
/// - Extract UserContext with user_id, store_id, and permissions
/// - Inject UserContext as extractor for handlers
#[derive(Debug, Clone)]
pub struct CurrentUser(pub UserContext);

impl CurrentUser {
    /// Returns a reference to the inner UserContext.
    pub fn context(&self) -> &UserContext {
        &self.0
    }

    /// Consumes the extractor and returns the inner UserContext.
    pub fn into_inner(self) -> UserContext {
        self.0
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserContext>()
            .cloned()
            .map(CurrentUser)
            .ok_or_else(|| {
                let error_response = ErrorResponse::new(
                    "UNAUTHORIZED",
                    "Authentication required. Please provide a valid token.",
                );
                (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use identity::{PermissionCode, StoreId, UserId};
    use std::collections::HashSet;

    fn create_test_context() -> UserContext {
        let permissions: HashSet<PermissionCode> = ["stores:read", "stores:create"]
            .iter()
            .filter_map(|p| PermissionCode::new(p).ok())
            .collect();
        UserContext::new(UserId::new(), StoreId::new(), permissions)
    }

    #[tokio::test]
    async fn test_current_user_extraction_success() {
        let ctx = create_test_context();
        let user_id = *ctx.user_id();
        let store_id = *ctx.store_id();

        // Create a request with UserContext in extensions
        let mut request = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();
        request.extensions_mut().insert(ctx);

        // Extract parts
        let (mut parts, _body) = request.into_parts();

        // Extract CurrentUser
        let result = CurrentUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let current_user = result.unwrap();
        assert_eq!(*current_user.0.user_id(), user_id);
        assert_eq!(*current_user.0.store_id(), store_id);
    }

    #[tokio::test]
    async fn test_current_user_extraction_missing_context() {
        // Create a request without UserContext
        let request = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();

        // Extract parts
        let (mut parts, _body) = request.into_parts();

        // Extract CurrentUser - should fail
        let result = CurrentUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_current_user_context_method() {
        let ctx = create_test_context();
        let current_user = CurrentUser(ctx.clone());

        assert_eq!(*current_user.context().user_id(), *ctx.user_id());
    }

    #[test]
    fn test_current_user_into_inner() {
        let ctx = create_test_context();
        let user_id = *ctx.user_id();
        let current_user = CurrentUser(ctx);

        let inner = current_user.into_inner();
        assert_eq!(*inner.user_id(), user_id);
    }

    #[test]
    fn test_current_user_has_permission() {
        let ctx = create_test_context();
        let current_user = CurrentUser(ctx);

        assert!(current_user.0.has_permission("stores:read"));
        assert!(current_user.0.has_permission("stores:create"));
        assert!(!current_user.0.has_permission("stores:delete"));
    }
}
