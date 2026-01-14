// Extractors for the API Gateway
//
// This module contains custom Axum extractors for use in handlers.

pub mod current_user;

pub use current_user::CurrentUser;
