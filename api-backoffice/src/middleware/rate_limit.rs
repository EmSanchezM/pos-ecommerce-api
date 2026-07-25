// Rate limiting for the backoffice binary
//
// Per-IP throttling via tower_governor. Returns 429 when the limit is exceeded.
//
// Why this exists: the backoffice login grants platform-owner access to EVERY
// organization. An unthrottled login endpoint is an open brute-force target
// with the highest-value credential in the system behind it.
//
// # Why PeerIpKeyExtractor and not SmartIpKeyExtractor
//
// `SmartIpKeyExtractor` reads `X-Forwarded-For` / `X-Real-IP` / `Forwarded`
// BEFORE falling back to the peer address. Those headers are set by the client,
// so on a directly-exposed service an attacker gets a fresh quota bucket per
// request just by rotating the header — the limiter stops limiting anything.
// tower_governor's own docs carry this warning: use it "only if you can ensure
// these headers are being set by a trusted provider".
//
// `compose.dev.yml` publishes the backoffice on :8001 with no proxy in front,
// so the peer address IS the client and is not forgeable. This mirrors the
// resolution order already used for the audit IP in `middleware/auth.rs`:
// trust the transport, treat client-supplied headers as untrusted.
//
// If a reverse proxy is ever put in front of this binary, `PeerIpKeyExtractor`
// starts seeing only the proxy address and throttles every operator as one
// bucket. The fix at that point is to switch to `SmartIpKeyExtractor` AND
// configure the proxy to overwrite (not append to) any client-supplied
// `X-Forwarded-For`. Switching without the proxy-side change reintroduces the
// bypass described above.
//
// Both layers depend on `ConnectInfo<SocketAddr>` being present, which
// `main` installs via `into_make_service_with_connect_info`.

use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

/// Seconds between quota replenishments on the login endpoint.
///
/// NOTE: `per_second` in tower_governor is the interval *between* replenished
/// elements, not a per-second rate — this is one token every 15s, i.e. 4
/// sustained attempts per minute per IP.
const LOGIN_REPLENISH_INTERVAL_SECS: u64 = 15;

/// Rapid login attempts allowed before throttling begins.
///
/// Enough for an operator to fumble a password a few times; far too few to
/// make an online brute force worth attempting.
const LOGIN_BURST_SIZE: u32 = 5;

/// Seconds between quota replenishments on the authenticated surface.
const API_REPLENISH_INTERVAL_SECS: u64 = 1;

/// Rapid authenticated requests allowed before throttling begins.
///
/// Deliberately generous: this is a blast-radius cap on a stolen or misused
/// backoffice token (cross-org scraping), not a limit a human operator or the
/// dev e2e flow should ever notice.
const API_BURST_SIZE: u32 = 60;

// Guardrails on the constants above. These are compile-time assertions on
// purpose: a bad edit fails the BUILD rather than a test someone can skip.

/// The login endpoint must stay strictly tighter than the general API — it
/// guards the highest-value credential in the system.
const _: () = assert!(LOGIN_BURST_SIZE < API_BURST_SIZE);
const _: () = assert!(LOGIN_REPLENISH_INTERVAL_SECS > API_REPLENISH_INTERVAL_SECS);

/// A burst large enough to keep online password guessing practical defeats the
/// point of the control.
const _: () = assert!(LOGIN_BURST_SIZE <= 10);

/// Sustained login attempts per hour per IP, as configured. Documents the
/// actual guarantee so a change to either constant is visible in review.
const _: () = assert!(3600 / LOGIN_REPLENISH_INTERVAL_SECS <= 300);

/// Rate-limiting layer for `POST /backoffice/auth/login`.
///
/// Burst of [`LOGIN_BURST_SIZE`], then one attempt every
/// [`LOGIN_REPLENISH_INTERVAL_SECS`] seconds, keyed by peer IP.
pub fn login_rate_limit_layer()
-> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware, axum::body::Body> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(LOGIN_REPLENISH_INTERVAL_SECS)
        .burst_size(LOGIN_BURST_SIZE)
        .finish()
        .expect("login GovernorConfig must be valid");

    GovernorLayer::new(config)
}

/// Rate-limiting layer for the authenticated backoffice surface.
///
/// Burst of [`API_BURST_SIZE`], then one request every
/// [`API_REPLENISH_INTERVAL_SECS`] second, keyed by peer IP.
pub fn api_rate_limit_layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware, axum::body::Body>
{
    let config = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(API_REPLENISH_INTERVAL_SECS)
        .burst_size(API_BURST_SIZE)
        .finish()
        .expect("api GovernorConfig must be valid");

    GovernorLayer::new(config)
}

// =============================================================================
// Tests
//
// The constants are guarded by the compile-time assertions above; the
// throttling behaviour itself is exercised end-to-end in `integration_tests`
// against the real router. What is left to check here is that both
// configurations are actually constructible — `finish()` returns an Option and
// an invalid quota would panic at startup, in production, on the first request.
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_layers_build() {
        let _login = login_rate_limit_layer();
        let _api = api_rate_limit_layer();
    }
}
