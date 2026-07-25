//! Client-IP resolution for rate limiting, aware of trusted reverse proxies.
//!
//! Both binaries run behind Caddy in production and are reachable directly in
//! development, so neither of tower_governor's built-in key extractors is
//! correct on its own:
//!
//! - `PeerIpKeyExtractor` sees only the proxy address in production and would
//!   throttle every operator as a single shared bucket.
//! - `SmartIpKeyExtractor` reads `X-Forwarded-For` and takes the FIRST
//!   parseable entry. Caddy's `reverse_proxy` APPENDS the observed client to
//!   any header the client already sent, so the first entry is attacker
//!   controlled — rotating it hands out a fresh quota bucket per request and
//!   the limiter stops limiting anything.
//!
//! # The rule
//!
//! A forwarded header is evidence only when the hop that delivered it is one we
//! trust. So:
//!
//! 1. If the peer is NOT a configured trusted proxy, use the peer address and
//!    ignore every header. A direct client cannot talk its way out of its own
//!    quota.
//! 2. If the peer IS trusted, walk `X-Forwarded-For` from the RIGHT, skipping
//!    entries that are themselves trusted proxies, and take the first entry
//!    that is not. With Caddy appending, the rightmost entry is what Caddy
//!    itself observed; everything to its left was supplied by the client.
//! 3. If that yields nothing usable, fall back to the peer address.
//!
//! Reading from the right is the whole point. Reading from the left is the bug
//! `SmartIpKeyExtractor` has.
//!
//! With no trusted proxies configured this degrades exactly to peer-IP keying,
//! which is the correct behaviour for a directly exposed service.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::ConnectInfo;
use http::request::Request;
use ipnetwork::IpNetwork;
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};

/// Header carrying the forwarded client chain.
const X_FORWARDED_FOR: &str = "x-forwarded-for";

/// A set of reverse proxies whose forwarded headers may be believed.
///
/// Empty means "trust nothing", which is the safe default: every request is
/// keyed on the address that actually connected.
#[derive(Clone, Debug, Default)]
pub struct TrustedProxies {
    networks: Arc<[IpNetwork]>,
}

impl TrustedProxies {
    /// Builds from parsed networks.
    pub fn new(networks: Vec<IpNetwork>) -> Self {
        Self {
            networks: networks.into(),
        }
    }

    /// Parses a comma-separated list of IPs and/or CIDR blocks.
    ///
    /// Bare addresses are accepted and treated as single-host networks, so
    /// `10.0.0.5` and `10.0.0.5/32` mean the same thing. Empty and
    /// whitespace-only entries are ignored so a trailing comma is harmless.
    pub fn parse(raw: &str) -> Result<Self, TrustedProxyParseError> {
        let mut networks = Vec::new();
        for entry in raw.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let network = entry
                .parse::<IpNetwork>()
                .map_err(|_| TrustedProxyParseError {
                    entry: entry.to_string(),
                })?;
            networks.push(network);
        }
        Ok(Self::new(networks))
    }

    /// Reads and parses an environment variable, panicking on a malformed
    /// value. An unset variable yields an empty set.
    ///
    /// Failing at startup is deliberate: silently falling back to "trust
    /// nothing" behind a proxy would collapse every client into one shared
    /// bucket, and the only symptom would be mysterious 429s in production.
    /// Both binaries load configuration this way.
    pub fn from_env(var: &str) -> Self {
        match std::env::var(var) {
            Ok(raw) => Self::parse(&raw).unwrap_or_else(|e| panic!("{var} is invalid: {e}")),
            Err(_) => Self::default(),
        }
    }

    /// True when no proxy is trusted — forwarded headers are then ignored
    /// entirely.
    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    /// True when `ip` falls inside any configured network.
    pub fn contains(&self, ip: IpAddr) -> bool {
        self.networks.iter().any(|net| net.contains(ip))
    }

    /// Resolves the address to key a rate limit on.
    ///
    /// See the module docs for the rule. `forwarded_for` is the raw
    /// `X-Forwarded-For` value, if present.
    pub fn resolve_client_ip(&self, peer: IpAddr, forwarded_for: Option<&str>) -> IpAddr {
        // A hop we do not trust gets no say in which bucket it lands in.
        if !self.contains(peer) {
            return peer;
        }

        let Some(chain) = forwarded_for else {
            return peer;
        };

        // Right to left: the rightmost entry is the one our trusted proxy
        // observed. Skip further trusted hops so a chain (CDN -> Caddy -> app)
        // still resolves to the real client rather than to the CDN.
        chain
            .split(',')
            .rev()
            .filter_map(|entry| entry.trim().parse::<IpAddr>().ok())
            .find(|ip| !self.contains(*ip))
            .unwrap_or(peer)
    }
}

/// A `TRUSTED_PROXY_IPS` entry that is neither an IP nor a CIDR block.
#[derive(Debug, Clone)]
pub struct TrustedProxyParseError {
    pub entry: String,
}

impl std::fmt::Display for TrustedProxyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'{}' is not a valid IP address or CIDR block",
            self.entry
        )
    }
}

impl std::error::Error for TrustedProxyParseError {}

/// tower_governor key extractor built on [`TrustedProxies`].
///
/// Requires `ConnectInfo<SocketAddr>` in the request extensions, which both
/// binaries install via `into_make_service_with_connect_info`. Without it the
/// request is rejected rather than silently sharing a global bucket — failing
/// closed is the right call for a control that exists to stop brute force.
#[derive(Clone, Debug, Default)]
pub struct ClientIpKeyExtractor {
    trusted: TrustedProxies,
}

impl ClientIpKeyExtractor {
    pub fn new(trusted: TrustedProxies) -> Self {
        Self { trusted }
    }
}

impl KeyExtractor for ClientIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let peer = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|info| info.0.ip())
            .ok_or(GovernorError::UnableToExtractKey)?;

        let forwarded_for = req
            .headers()
            .get(X_FORWARDED_FOR)
            .and_then(|value| value.to_str().ok());

        Ok(self.trusted.resolve_client_ip(peer, forwarded_for))
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn caddy_only() -> TrustedProxies {
        TrustedProxies::parse("10.0.0.1").unwrap()
    }

    // --- parsing ---------------------------------------------------------

    #[test]
    fn parses_bare_addresses_and_cidr_blocks() {
        let trusted = TrustedProxies::parse("10.0.0.1, 172.16.0.0/12, ::1").unwrap();
        assert!(trusted.contains(ip("10.0.0.1")));
        assert!(trusted.contains(ip("172.20.5.7")));
        assert!(trusted.contains(ip("::1")));
        assert!(!trusted.contains(ip("10.0.0.2")));
        assert!(!trusted.contains(ip("192.168.1.1")));
    }

    #[test]
    fn empty_configuration_trusts_nothing() {
        let trusted = TrustedProxies::parse("").unwrap();
        assert!(trusted.is_empty());
        assert!(!trusted.contains(ip("10.0.0.1")));
    }

    /// Trailing commas and stray whitespace are common in env files.
    #[test]
    fn blank_entries_are_ignored() {
        let trusted = TrustedProxies::parse(" 10.0.0.1 , , ").unwrap();
        assert!(trusted.contains(ip("10.0.0.1")));
    }

    /// A typo must surface at startup, not silently disable the proxy logic.
    #[test]
    fn malformed_entry_is_an_error() {
        let err = TrustedProxies::parse("10.0.0.1, not-an-ip").unwrap_err();
        assert_eq!(err.entry, "not-an-ip");
    }

    // --- resolution ------------------------------------------------------

    /// No trusted proxies configured: behaves exactly like peer-IP keying.
    #[test]
    fn untrusted_peer_is_keyed_on_its_own_address() {
        let trusted = TrustedProxies::default();
        let resolved = trusted.resolve_client_ip(ip("203.0.113.9"), Some("1.2.3.4"));
        assert_eq!(resolved, ip("203.0.113.9"));
    }

    /// THE attack: a direct client sends a forged header. It must be ignored
    /// outright, because the hop that delivered it is not trusted.
    #[test]
    fn forged_header_from_a_direct_client_is_ignored() {
        let trusted = caddy_only();
        let resolved = trusted.resolve_client_ip(ip("203.0.113.9"), Some("9.9.9.9, 8.8.8.8"));
        assert_eq!(
            resolved,
            ip("203.0.113.9"),
            "a header from an untrusted peer must never choose the bucket"
        );
    }

    /// Behind Caddy: the rightmost entry is what Caddy observed.
    #[test]
    fn trusted_proxy_yields_the_rightmost_entry() {
        let trusted = caddy_only();
        let resolved = trusted.resolve_client_ip(ip("10.0.0.1"), Some("203.0.113.9"));
        assert_eq!(resolved, ip("203.0.113.9"));
    }

    /// THE bug in SmartIpKeyExtractor: Caddy APPENDS, so a client-supplied
    /// value sits to the LEFT of the real address. Taking the first entry
    /// would hand the attacker a bucket of their choosing.
    #[test]
    fn client_supplied_prefix_does_not_win_behind_a_trusted_proxy() {
        let trusted = caddy_only();
        let resolved = trusted.resolve_client_ip(ip("10.0.0.1"), Some("1.2.3.4, 203.0.113.9"));
        assert_eq!(
            resolved,
            ip("203.0.113.9"),
            "must take the entry the trusted proxy appended, not the client's"
        );
        assert_ne!(resolved, ip("1.2.3.4"));
    }

    /// Rotating the forged prefix must not move the bucket at all.
    #[test]
    fn rotating_a_forged_prefix_keeps_the_same_key() {
        let trusted = caddy_only();
        let first = trusted.resolve_client_ip(ip("10.0.0.1"), Some("1.1.1.1, 203.0.113.9"));
        let second = trusted.resolve_client_ip(ip("10.0.0.1"), Some("2.2.2.2, 203.0.113.9"));
        let third = trusted.resolve_client_ip(ip("10.0.0.1"), Some("3.3.3.3, 203.0.113.9"));
        assert_eq!(first, second);
        assert_eq!(second, third);
        assert_eq!(first, ip("203.0.113.9"));
    }

    /// A chain of trusted hops (CDN -> Caddy -> app) resolves past both.
    #[test]
    fn chained_trusted_hops_are_skipped() {
        let trusted = TrustedProxies::parse("10.0.0.1, 10.0.0.2").unwrap();
        let resolved =
            trusted.resolve_client_ip(ip("10.0.0.1"), Some("203.0.113.9, 10.0.0.2, 10.0.0.1"));
        assert_eq!(resolved, ip("203.0.113.9"));
    }

    /// A trusted peer that sent no header falls back to the peer address.
    #[test]
    fn trusted_peer_without_a_header_falls_back_to_peer() {
        let trusted = caddy_only();
        assert_eq!(
            trusted.resolve_client_ip(ip("10.0.0.1"), None),
            ip("10.0.0.1")
        );
    }

    /// Garbage in the header must not panic or produce a bogus key.
    #[test]
    fn unparseable_header_falls_back_to_peer() {
        let trusted = caddy_only();
        assert_eq!(
            trusted.resolve_client_ip(ip("10.0.0.1"), Some("not-an-ip, also-garbage")),
            ip("10.0.0.1")
        );
    }

    /// An all-trusted chain means we never saw an external client; keying on
    /// the peer is the only honest answer.
    #[test]
    fn chain_of_only_trusted_hops_falls_back_to_peer() {
        let trusted = TrustedProxies::parse("10.0.0.1, 10.0.0.2").unwrap();
        assert_eq!(
            trusted.resolve_client_ip(ip("10.0.0.1"), Some("10.0.0.2, 10.0.0.1")),
            ip("10.0.0.1")
        );
    }

    /// Whitespace around entries is normal in a forwarded chain.
    #[test]
    fn chain_entries_are_trimmed() {
        let trusted = caddy_only();
        assert_eq!(
            trusted.resolve_client_ip(ip("10.0.0.1"), Some("  1.2.3.4 ,  203.0.113.9  ")),
            ip("203.0.113.9")
        );
    }

    /// Proxies reached over a CIDR range (docker networks) behave the same.
    #[test]
    fn cidr_configured_proxy_is_trusted() {
        let trusted = TrustedProxies::parse("172.16.0.0/12").unwrap();
        let resolved = trusted.resolve_client_ip(ip("172.20.0.3"), Some("1.2.3.4, 203.0.113.9"));
        assert_eq!(resolved, ip("203.0.113.9"));
    }

    #[test]
    fn ipv6_clients_resolve_through_a_trusted_proxy() {
        let trusted = caddy_only();
        let resolved = trusted.resolve_client_ip(ip("10.0.0.1"), Some("2001:db8::1"));
        assert_eq!(resolved, ip("2001:db8::1"));
    }
}
