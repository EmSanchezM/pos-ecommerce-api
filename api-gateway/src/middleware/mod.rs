// Middleware modules for the API Gateway
//
// This module contains authentication and authorization middleware
// for protecting API endpoints.

pub mod auth;
pub mod permission;

pub use auth::auth_middleware;
pub use permission::{require_permission, require_all_permissions, require_any_permission, require_super_admin};
