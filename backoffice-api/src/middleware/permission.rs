// Backoffice Permission Guards
//
// Mirrors api-gateway/src/middleware/permission.rs but for BackofficeUserContext.
// Used by handlers after auth middleware has injected the context.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::ErrorResponse;
use crate::middleware::auth::BackofficeUserContext;

/// Asserts that the backoffice user has the specified platform permission.
///
/// Returns `Ok(())` if the user has the permission, `Err(Response)` with
/// HTTP 403 if the permission is missing.
///
/// # Example
///
/// ```ignore
/// pub async fn list_orgs(
///     Extension(ctx): Extension<BackofficeUserContext>,
/// ) -> Result<Json<OrgList>, Response> {
///     require_backoffice_permission(&ctx, "platform:org.list")?;
///     // ...
/// }
/// ```
#[allow(clippy::result_large_err)]
pub fn require_backoffice_permission(
    ctx: &BackofficeUserContext,
    permission: &str,
) -> Result<(), Response> {
    if ctx.has_permission(permission) {
        Ok(())
    } else {
        Err(forbidden(&format!(
            "Missing required platform permission: {permission}"
        )))
    }
}

fn forbidden(message: &str) -> Response {
    let body = ErrorResponse::forbidden(message);
    (StatusCode::FORBIDDEN, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::BackofficeUserContext;
    use uuid::Uuid;

    fn test_uuid() -> Uuid {
        use uuid::{NoContext, Timestamp};
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    fn ctx_with(perms: &[&str]) -> BackofficeUserContext {
        BackofficeUserContext::new(
            test_uuid(),
            perms.iter().map(|p| p.to_string()).collect(),
        )
    }

    /// P3-T05: user with permission passes gate.
    #[test]
    fn user_with_permission_passes() {
        let ctx = ctx_with(&["platform:org.list", "platform:org.suspend"]);
        assert!(require_backoffice_permission(&ctx, "platform:org.list").is_ok());
        assert!(require_backoffice_permission(&ctx, "platform:org.suspend").is_ok());
    }

    /// P3-T05: user without required permission is denied with 403.
    #[test]
    fn user_without_permission_is_denied() {
        let ctx = ctx_with(&["platform:org.list"]);
        let result = require_backoffice_permission(&ctx, "platform:user.impersonate");
        assert!(result.is_err());
    }

    /// P3-T05: user with no permissions is denied.
    #[test]
    fn user_with_no_permissions_is_denied() {
        let ctx = ctx_with(&[]);
        let result = require_backoffice_permission(&ctx, "platform:org.list");
        assert!(result.is_err());
    }
}
