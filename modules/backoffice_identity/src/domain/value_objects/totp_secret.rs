//! The shared secret behind a TOTP authenticator.
//!
//! Wrapped rather than passed as a `String` so it cannot be logged, compared or
//! serialized by accident: this value IS the second factor, and anything
//! holding it can generate valid codes indefinitely.

use crate::error::BackofficeIdentityError;

/// RFC 4648 base32 alphabet, which is what authenticator apps expect.
const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Minimum length accepted. RFC 4226 requires at least 128 bits of shared
/// secret; 26 base32 characters carry 130.
const MIN_BASE32_LENGTH: usize = 26;

/// Guards the storage column (`VARCHAR(255)`) against an oversized value.
const MAX_BASE32_LENGTH: usize = 255;

/// A validated base32 TOTP shared secret.
///
/// `Debug` is implemented manually to redact the value — a derived `Debug` on a
/// struct that ends up in a log line or an error chain would publish the second
/// factor in plain text.
#[derive(Clone, PartialEq, Eq)]
pub struct TotpSecret(String);

impl TotpSecret {
    /// Validates and wraps a base32 secret.
    ///
    /// Rejects padding (`=`): authenticator provisioning URIs carry unpadded
    /// base32, and accepting both would let the same secret exist under two
    /// spellings.
    pub fn new(value: impl Into<String>) -> Result<Self, BackofficeIdentityError> {
        let value = value.into();

        if value.len() < MIN_BASE32_LENGTH || value.len() > MAX_BASE32_LENGTH {
            return Err(BackofficeIdentityError::InvalidMfaSecret);
        }

        if !value
            .bytes()
            .all(|byte| BASE32_ALPHABET.contains(&byte.to_ascii_uppercase()))
        {
            return Err(BackofficeIdentityError::InvalidMfaSecret);
        }

        Ok(Self(value.to_ascii_uppercase()))
    }

    /// The base32 representation, for the TOTP implementation and for storage.
    ///
    /// Named to make call sites read as a deliberate disclosure.
    pub fn expose_base32(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for TotpSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TotpSecret(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "JBSWY3DPEHPK3PXPJBSWY3DPEH";

    #[test]
    fn accepts_a_valid_base32_secret() {
        let secret = TotpSecret::new(VALID).expect("should accept valid base32");
        assert_eq!(secret.expose_base32(), VALID);
    }

    #[test]
    fn normalizes_to_uppercase() {
        let secret = TotpSecret::new(VALID.to_lowercase()).unwrap();
        assert_eq!(secret.expose_base32(), VALID);
    }

    #[test]
    fn rejects_a_short_secret() {
        assert!(TotpSecret::new("JBSWY3DP").is_err());
    }

    #[test]
    fn rejects_an_empty_secret() {
        assert!(TotpSecret::new("").is_err());
    }

    #[test]
    fn rejects_an_oversized_secret() {
        assert!(TotpSecret::new("A".repeat(MAX_BASE32_LENGTH + 1)).is_err());
    }

    /// `0`, `1` and `8` are not in the base32 alphabet.
    #[test]
    fn rejects_characters_outside_the_base32_alphabet() {
        assert!(TotpSecret::new("JBSWY3DPEHPK3PXPJBSWY3DPE0").is_err());
        assert!(TotpSecret::new("JBSWY3DPEHPK3PXPJBSWY3DPE1").is_err());
        assert!(TotpSecret::new("JBSWY3DPEHPK3PXPJBSWY3DPE8").is_err());
    }

    /// Padded base32 would give the same secret two spellings.
    #[test]
    fn rejects_padding() {
        assert!(TotpSecret::new("JBSWY3DPEHPK3PXPJBSWY3DPE=").is_err());
    }

    /// The secret must never reach a log line through `Debug`.
    #[test]
    fn debug_output_redacts_the_secret() {
        let secret = TotpSecret::new(VALID).unwrap();
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains(VALID), "Debug leaked the secret");
        assert_eq!(rendered, "TotpSecret(redacted)");
    }

    /// The same guarantee inside a container, which is how it usually escapes.
    #[test]
    fn debug_output_redacts_inside_a_container() {
        let secret = TotpSecret::new(VALID).unwrap();
        let rendered = format!("{:?}", Some(secret));
        assert!(!rendered.contains(VALID), "Debug leaked the secret");
    }
}
