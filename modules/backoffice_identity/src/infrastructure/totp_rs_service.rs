//! RFC 6238 TOTP via the `totp-rs` crate.
//!
//! Parameters are the ones every authenticator app assumes by default —
//! SHA-1, 6 digits, 30-second step. They are not a security choice so much as
//! an interoperability one: changing any of them makes previously enrolled
//! authenticators produce codes this service rejects.

use std::time::{SystemTime, UNIX_EPOCH};

use totp_rs::{Algorithm, Secret, TOTP};

use crate::domain::auth::TotpService;
use crate::domain::value_objects::TotpSecret;
use crate::error::BackofficeIdentityError;

/// Digits per code, as every authenticator app expects.
const DIGITS: usize = 6;

/// Seconds per TOTP step.
const STEP_SECONDS: u64 = 30;

/// Steps of clock drift tolerated either side of now.
///
/// One step means a code stays usable for up to ~90 seconds across the window
/// boundary. Larger values forgive worse clocks at the cost of widening the
/// window an observed code can be replayed in — which is exactly why
/// verification also records the step and rejects repeats.
const SKEW: u8 = 1;

/// Shown by the authenticator app above the code.
const ISSUER: &str = "POS Ecommerce Backoffice";

pub struct TotpRsService {
    issuer: String,
}

impl TotpRsService {
    pub fn new() -> Self {
        Self {
            issuer: ISSUER.to_string(),
        }
    }

    /// Overrides the issuer label, mainly so tests do not depend on the
    /// production string.
    pub fn with_issuer(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
        }
    }

    /// Builds the `totp-rs` handle for a secret and account.
    fn totp(
        &self,
        secret: &TotpSecret,
        account_label: &str,
    ) -> Result<TOTP, BackofficeIdentityError> {
        let bytes = Secret::Encoded(secret.expose_base32().to_string())
            .to_bytes()
            .map_err(|e| BackofficeIdentityError::MfaError(format!("invalid secret: {e}")))?;

        TOTP::new(
            Algorithm::SHA1,
            DIGITS,
            SKEW,
            STEP_SECONDS,
            bytes,
            Some(self.issuer.clone()),
            account_label.to_string(),
        )
        .map_err(|e| BackofficeIdentityError::MfaError(format!("invalid TOTP parameters: {e}")))
    }

    fn now_secs() -> Result<u64, BackofficeIdentityError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|e| {
                BackofficeIdentityError::MfaError(format!("system clock is before epoch: {e}"))
            })
    }
}

impl Default for TotpRsService {
    fn default() -> Self {
        Self::new()
    }
}

impl TotpService for TotpRsService {
    fn generate_secret(&self) -> Result<TotpSecret, BackofficeIdentityError> {
        let encoded = Secret::generate_secret()
            .to_encoded()
            .to_string()
            // `to_encoded` emits padded base32; the domain type rejects padding
            // so a secret has exactly one spelling.
            .replace('=', "");

        TotpSecret::new(encoded)
    }

    fn provisioning_uri(
        &self,
        secret: &TotpSecret,
        account_label: &str,
    ) -> Result<String, BackofficeIdentityError> {
        Ok(self.totp(secret, account_label)?.get_url())
    }

    fn verify(
        &self,
        secret: &TotpSecret,
        code: &str,
    ) -> Result<Option<u64>, BackofficeIdentityError> {
        // Reject obvious non-codes before doing any crypto. `check_current`
        // would return false anyway; this keeps the work proportional.
        if code.len() != DIGITS || !code.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(None);
        }

        let totp = self.totp(secret, "verify")?;
        let current_step = Self::now_secs()? / STEP_SECONDS;
        let skew = u64::from(SKEW);

        // Walk the accepted window and report WHICH step matched. `totp-rs`
        // only answers yes/no, and the step is what makes replay rejection
        // possible upstream.
        for step in current_step.saturating_sub(skew)..=current_step + skew {
            let expected = totp.generate(step * STEP_SECONDS);
            if constant_time_eq(expected.as_bytes(), code.as_bytes()) {
                return Ok(Some(step));
            }
        }

        Ok(None)
    }
}

/// Length-aware constant-time comparison, so a wrong code cannot be narrowed
/// down by timing. Mirrors the helper in `api-gateway`'s internal handler.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> TotpRsService {
        TotpRsService::with_issuer("Test Issuer")
    }

    /// The generated secret must satisfy the domain type — including the
    /// no-padding rule, which `to_encoded` does not honour on its own.
    #[test]
    fn generated_secrets_are_valid_domain_secrets() {
        let secret = service()
            .generate_secret()
            .expect("generation must succeed");
        assert!(!secret.expose_base32().contains('='));
        TotpSecret::new(secret.expose_base32()).expect("must round-trip through validation");
    }

    #[test]
    fn generated_secrets_differ() {
        let svc = service();
        let first = svc.generate_secret().unwrap();
        let second = svc.generate_secret().unwrap();
        assert_ne!(first.expose_base32(), second.expose_base32());
    }

    #[test]
    fn provisioning_uri_carries_issuer_and_account() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();
        let uri = svc.provisioning_uri(&secret, "admin@platform.com").unwrap();

        assert!(uri.starts_with("otpauth://totp/"), "got {uri}");
        assert!(uri.contains("Test%20Issuer"), "issuer missing from {uri}");
        assert!(
            uri.contains("admin%40platform.com"),
            "account missing from {uri}"
        );
        // `digits` and `period` are absent on purpose: totp-rs omits parameters
        // that match the otpauth defaults (6 digits, 30s), and those defaults
        // are exactly what DIGITS and STEP_SECONDS are set to. If either
        // constant changes, they must start appearing here — and every already
        // enrolled authenticator breaks.
        assert!(!uri.contains("digits="), "non-default digits in {uri}");
        assert!(!uri.contains("period="), "non-default period in {uri}");
    }

    /// The provisioning URI necessarily contains the secret — that is its job —
    /// so the only protection is that it is shown once, over TLS.
    #[test]
    fn provisioning_uri_contains_the_secret() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();
        let uri = svc.provisioning_uri(&secret, "admin@platform.com").unwrap();
        assert!(uri.contains(secret.expose_base32()));
    }

    #[test]
    fn a_freshly_generated_code_verifies_and_reports_its_step() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();

        let totp = svc.totp(&secret, "verify").unwrap();
        let now = TotpRsService::now_secs().unwrap();
        let code = totp.generate(now);

        let step = svc.verify(&secret, &code).unwrap();
        assert_eq!(step, Some(now / STEP_SECONDS));
    }

    #[test]
    fn a_wrong_code_does_not_verify() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();

        let totp = svc.totp(&secret, "verify").unwrap();
        let now = TotpRsService::now_secs().unwrap();
        let correct = totp.generate(now);
        // Perturb one digit so length and shape stay valid.
        let wrong: String = correct
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    if c == '0' { '1' } else { '0' }
                } else {
                    c
                }
            })
            .collect();

        assert_eq!(svc.verify(&secret, &wrong).unwrap(), None);
    }

    /// A code from another secret must never verify.
    #[test]
    fn a_code_from_a_different_secret_does_not_verify() {
        let svc = service();
        let mine = svc.generate_secret().unwrap();
        let theirs = svc.generate_secret().unwrap();

        let now = TotpRsService::now_secs().unwrap();
        let their_code = svc.totp(&theirs, "verify").unwrap().generate(now);

        assert_eq!(svc.verify(&mine, &their_code).unwrap(), None);
    }

    /// Clock drift within the skew window is tolerated — and reports the step
    /// it actually matched, not the current one.
    #[test]
    fn a_code_from_the_previous_step_still_verifies() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();

        let now = TotpRsService::now_secs().unwrap();
        let previous_step = (now / STEP_SECONDS) - 1;
        let code = svc
            .totp(&secret, "verify")
            .unwrap()
            .generate(previous_step * STEP_SECONDS);

        assert_eq!(svc.verify(&secret, &code).unwrap(), Some(previous_step));
    }

    /// Outside the window it must fail, or the replay watermark buys nothing.
    #[test]
    fn a_code_from_far_in_the_past_does_not_verify() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();

        let now = TotpRsService::now_secs().unwrap();
        let stale_step = (now / STEP_SECONDS) - 10;
        let code = svc
            .totp(&secret, "verify")
            .unwrap()
            .generate(stale_step * STEP_SECONDS);

        assert_eq!(svc.verify(&secret, &code).unwrap(), None);
    }

    // --- malformed input -----------------------------------------------------

    #[test]
    fn malformed_codes_are_rejected_without_error() {
        let svc = service();
        let secret = svc.generate_secret().unwrap();

        for bad in ["", "12345", "1234567", "abcdef", "12 456", "12345a"] {
            assert_eq!(
                svc.verify(&secret, bad).unwrap(),
                None,
                "{bad:?} must be rejected as a wrong code, not an error"
            );
        }
    }

    #[test]
    fn constant_time_eq_matches_only_identical_slices() {
        assert!(constant_time_eq(b"123456", b"123456"));
        assert!(!constant_time_eq(b"123456", b"123457"));
        assert!(!constant_time_eq(b"123456", b"12345"));
        assert!(!constant_time_eq(b"", b"0"));
        assert!(constant_time_eq(b"", b""));
    }
}
