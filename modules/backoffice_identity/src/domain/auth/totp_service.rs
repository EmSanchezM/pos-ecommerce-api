//! Port for TOTP generation and verification.
//!
//! The domain states what it needs from a second factor; the concrete RFC 6238
//! implementation lives in infrastructure. Use cases depend on this trait, so
//! enrollment and verification logic is testable without a real clock or a real
//! TOTP library.

use crate::domain::value_objects::TotpSecret;
use crate::error::BackofficeIdentityError;

/// Generates and verifies time-based one-time passwords.
pub trait TotpService: Send + Sync {
    /// Produces a fresh random shared secret.
    fn generate_secret(&self) -> Result<TotpSecret, BackofficeIdentityError>;

    /// Builds the `otpauth://` URI an authenticator app consumes, usually via
    /// a QR code.
    ///
    /// `account_label` identifies the account inside the app — the operator's
    /// email, so someone holding several is not left guessing.
    fn provisioning_uri(
        &self,
        secret: &TotpSecret,
        account_label: &str,
    ) -> Result<String, BackofficeIdentityError>;

    /// Checks `code` against `secret` at the current time.
    ///
    /// Returns the TOTP step counter the code belongs to, which the caller
    /// records to reject replays. `Ok(None)` means the code is simply wrong —
    /// that is an expected outcome, not an error.
    fn verify(
        &self,
        secret: &TotpSecret,
        code: &str,
    ) -> Result<Option<u64>, BackofficeIdentityError>;
}
