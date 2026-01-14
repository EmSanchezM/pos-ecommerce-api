// Permission Guards for Authorization
//
// This module provides helper functions for checking user permissions
// in handlers. These are used after the auth middleware has injected
// the UserContext into the request.

use axum::{http::StatusCode, Json, response::{IntoResponse, Response}};
use identity::{ErrorResponse, UserContext};

/// Checks if the user has a specific permission.
///
/// Returns Ok(()) if the user has the permission, or an error response
/// with 403 Forbidden if the permission is missing.
///
/// # Arguments
///
/// * `ctx` - The UserContext containing the user's permissions
/// * `permission` - The permission code to check (format: "module:action")
///
/// # Returns
///
/// * `Ok(())` - If the user has the permission
/// * `Err(Response)` - 403 Forbidden response if permission is missing
///
/// # Example
///
/// ```ignore
/// pub async fn create_store(
///     CurrentUser(ctx): CurrentUser,
///     // ...
/// ) -> Result<Json<StoreResponse>, Response> {
///     require_permission(&ctx, "stores:create")?;
///     // ... handler logic
/// }
/// ```
///
/// - Return 403 Forbidden if permission is missing
pub fn require_permission(ctx: &UserContext, permission: &str) -> Result<(), Response> {
    if ctx.has_permission(permission) {
        Ok(())
    } else {
        Err(forbidden_response(&format!(
            "Missing required permission: {}",
            permission
        )))
    }
}

/// Checks if the user has ALL of the specified permissions (AND logic).
///
/// Returns Ok(()) if the user has all permissions, or an error response
/// with 403 Forbidden listing the missing permissions.
///
/// # Arguments
///
/// * `ctx` - The UserContext containing the user's permissions
/// * `permissions` - A slice of permission codes to check
///
/// # Returns
///
/// * `Ok(())` - If the user has all permissions
/// * `Err(Response)` - 403 Forbidden response with missing permissions
///
/// # Example
///
/// ```ignore
/// pub async fn manage_store(
///     CurrentUser(ctx): CurrentUser,
///     // ...
/// ) -> Result<Json<StoreResponse>, Response> {
///     require_all_permissions(&ctx, &["stores:read", "stores:update"])?;
///     // ... handler logic
/// }
/// ```
///
/// - Return 403 Forbidden if any permission is missing
pub fn require_all_permissions(ctx: &UserContext, permissions: &[&str]) -> Result<(), Response> {
    let missing: Vec<&str> = permissions
        .iter()
        .filter(|p| !ctx.has_permission(p))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(forbidden_response(&format!(
            "Missing required permissions: {}",
            missing.join(", ")
        )))
    }
}

/// Checks if the user has AT LEAST ONE of the specified permissions (OR logic).
///
/// Returns Ok(()) if the user has at least one permission, or an error response
/// with 403 Forbidden if none of the permissions are present.
///
/// # Arguments
///
/// * `ctx` - The UserContext containing the user's permissions
/// * `permissions` - A slice of permission codes to check
///
/// # Returns
///
/// * `Ok(())` - If the user has at least one permission
/// * `Err(Response)` - 403 Forbidden response if no permissions match
///
/// # Example
///
/// ```ignore
/// pub async fn view_reports(
///     CurrentUser(ctx): CurrentUser,
///     // ...
/// ) -> Result<Json<ReportResponse>, Response> {
///     require_any_permission(&ctx, &["reports:view", "admin:all"])?;
///     // ... handler logic
/// }
/// ```
///
/// - Return 403 Forbidden if no permission matches
pub fn require_any_permission(ctx: &UserContext, permissions: &[&str]) -> Result<(), Response> {
    if permissions.is_empty() {
        return Err(forbidden_response("No permissions specified"));
    }

    if ctx.has_any_permission(permissions) {
        Ok(())
    } else {
        Err(forbidden_response(&format!(
            "Requires at least one of: {}",
            permissions.join(", ")
        )))
    }
}

/// Checks if the user has the super_admin role.
///
/// Super admin is identified by having the "system:admin" permission,
/// which is only granted to users with the super_admin role.
///
/// # Arguments
///
/// * `ctx` - The UserContext containing the user's permissions
///
/// # Returns
///
/// * `Ok(())` - If the user is a super admin
/// * `Err(Response)` - 403 Forbidden response if not a super admin
///
/// # Example
///
/// ```ignore
/// pub async fn create_store(
///     CurrentUser(ctx): CurrentUser,
///     // ...
/// ) -> Result<Json<StoreResponse>, Response> {
///     require_super_admin(&ctx)?;
///     // ... handler logic
/// }
/// ```
///
/// - Return 403 Forbidden with descriptive message if not super_admin
/// - Only super_admin can create stores
/// - Only super_admin can create terminals
pub fn require_super_admin(ctx: &UserContext) -> Result<(), Response> {
    // super_admin is identified by having the "system:admin" permission
    // This permission is only granted to users with the super_admin role
    if ctx.has_permission("system:admin") {
        Ok(())
    } else {
        Err(forbidden_response(
            "This operation requires super_admin role",
        ))
    }
}

/// Creates a 403 Forbidden response with a JSON error body.
///
/// # Arguments
///
/// * `message` - The error message to include in the response
///
/// # Returns
///
/// A Response with status 403 and JSON error body
fn forbidden_response(message: &str) -> Response {
    let error_response = ErrorResponse::new("FORBIDDEN", message);
    (StatusCode::FORBIDDEN, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use identity::{PermissionCode, StoreId, UserId};
    use std::collections::HashSet;

    fn create_context_with_permissions(permissions: &[&str]) -> UserContext {
        let perms: HashSet<PermissionCode> = permissions
            .iter()
            .filter_map(|p| PermissionCode::new(p).ok())
            .collect();
        UserContext::new(UserId::new(), StoreId::new(), perms)
    }

    #[test]
    fn test_require_permission_success() {
        let ctx = create_context_with_permissions(&["stores:create", "stores:read"]);
        let result = require_permission(&ctx, "stores:create");
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_permission_failure() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_permission(&ctx, "stores:create");
        assert!(result.is_err());
    }

    #[test]
    fn test_require_all_permissions_success() {
        let ctx = create_context_with_permissions(&["stores:create", "stores:read", "stores:update"]);
        let result = require_all_permissions(&ctx, &["stores:create", "stores:read"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_all_permissions_partial_failure() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_all_permissions(&ctx, &["stores:create", "stores:read"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_all_permissions_empty_list() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_all_permissions(&ctx, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_any_permission_success() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_any_permission(&ctx, &["stores:create", "stores:read"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_any_permission_failure() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_any_permission(&ctx, &["stores:create", "stores:delete"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_any_permission_empty_list() {
        let ctx = create_context_with_permissions(&["stores:read"]);
        let result = require_any_permission(&ctx, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_super_admin_success() {
        let ctx = create_context_with_permissions(&["system:admin"]);
        let result = require_super_admin(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_super_admin_failure() {
        let ctx = create_context_with_permissions(&["stores:create", "stores:read"]);
        let result = require_super_admin(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_super_admin_empty_permissions() {
        let ctx = create_context_with_permissions(&[]);
        let result = require_super_admin(&ctx);
        assert!(result.is_err());
    }
}
