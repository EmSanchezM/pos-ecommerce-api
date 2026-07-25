// Rate limiting middleware for authentication endpoints
//
// Applies per-client rate limiting. Returns 429 Too Many Requests when the
// limit is exceeded.
//
// Keying goes through `common::ClientIpKeyExtractor`, which resolves the client
// address against the configured `TRUSTED_PROXY_IPS`.
//
// This previously used `SmartIpKeyExtractor`, which takes the FIRST parseable
// entry of `X-Forwarded-For`. Caddy's `reverse_proxy` APPENDS the address it
// observed to any header the client already sent, so that first entry is
// attacker controlled: rotating it handed out a fresh quota bucket per request
// and the limiter stopped limiting anything. See the module docs on
// `common::rate_limit`.

use common::{ClientIpKeyExtractor, TrustedProxies};
use governor::middleware::NoOpMiddleware;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

/// Creates a rate-limiting layer for authentication endpoints.
///
/// Allows a burst of 10 requests, then replenishes ONE request every 6 seconds
/// (~10 sustained requests per minute per client).
///
/// NOTE: `per_second(n)` in tower_governor is the interval *between*
/// replenished elements, not a per-second rate. This doc comment previously
/// read "~6 req/s sustained", overstating the allowance by 36x — the timing
/// config itself was, and remains, unchanged.
pub fn auth_rate_limit_layer(
    trusted_proxies: TrustedProxies,
) -> GovernorLayer<ClientIpKeyExtractor, NoOpMiddleware, axum::body::Body> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(ClientIpKeyExtractor::new(trusted_proxies))
        .per_second(6)
        .burst_size(10)
        .finish()
        .expect("GovernorConfig must be valid");

    GovernorLayer::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_builds_without_trusted_proxies() {
        let _layer = auth_rate_limit_layer(TrustedProxies::default());
    }

    /// The production path: reached through a trusted proxy.
    #[test]
    fn layer_builds_with_trusted_proxies() {
        let trusted = TrustedProxies::parse("172.16.0.0/12").unwrap();
        let _layer = auth_rate_limit_layer(trusted);
    }
}
