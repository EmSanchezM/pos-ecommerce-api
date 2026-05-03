// Middleware modules for the API Gateway
//
// This module contains authentication and authorization middleware
// for protecting API endpoints.

pub mod auth;
pub mod org_scope;
pub mod permission;
pub mod rate_limit;

pub use auth::auth_middleware;
pub use org_scope::{require_feature, require_org_match, verify_store_in_org};
pub use permission::{
    require_all_permissions, require_any_permission, require_permission, require_super_admin,
};
