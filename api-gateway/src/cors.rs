use axum::http::{HeaderName, Method};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

use crate::config::AppConfig;

pub fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let methods = AllowMethods::from(vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ]);

    let headers = AllowHeaders::from(vec![
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-store-id"),
    ]);

    let origins = match &config.cors_allowed_origins {
        Some(origins_str) => {
            let parsed: Vec<_> = origins_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            AllowOrigin::list(parsed)
        }
        None => AllowOrigin::any(),
    };

    CorsLayer::new()
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_origin(origins)
}
