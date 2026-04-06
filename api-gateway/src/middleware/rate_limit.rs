// Rate limiting middleware for authentication endpoints
//
// Uses tower_governor to apply per-IP rate limiting.
// Returns 429 Too Many Requests when the limit is exceeded.

use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};

/// Creates a rate-limiting layer for authentication endpoints.
///
/// Allows a burst of 10 requests, refilling at ~6 tokens/second (i.e. roughly
/// 10 rapid requests before being throttled, then sustained ~6 req/s).
pub fn auth_rate_limit_layer()
-> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware, axum::body::Body> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(6)
        .burst_size(10)
        .finish()
        .expect("GovernorConfig must be valid");

    GovernorLayer::new(config)
}
