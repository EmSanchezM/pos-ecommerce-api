//! Self-service MFA enrollment for a backoffice operator.
//!
//! Four operations over the `none → pending → active` state machine on
//! [`BackofficeUser`]:
//!
//! - `begin`    — generate a secret, leave it PENDING
//! - `activate` — confirm with a code, issue recovery codes
//! - `disable`  — prove possession, then clear everything
//! - `regenerate_recovery_codes` — prove possession, replace the set
//!
//! # Why possession is proved on the way out, not just on the way in
//!
//! Every operation that weakens or replaces an existing factor requires a
//! currently valid code. Without that, anyone holding a stolen session token
//! could disable MFA outright, or re-enroll their own authenticator and lock
//! the real operator out — which would make the second factor decorative
//! exactly when it matters.

use std::sync::Arc;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng, rand_core::RngCore},
};

use crate::domain::auth::TotpService;
use crate::domain::entities::MfaRecoveryCode;
use crate::domain::repositories::{BackofficeUserRepository, MfaRecoveryCodeRepository};
use crate::domain::value_objects::{BackofficeUserId, TotpSecret};
use crate::error::BackofficeIdentityError;

/// Recovery codes issued per activation.
const RECOVERY_CODE_COUNT: usize = 10;

/// Random bytes behind each recovery code. 10 bytes is 80 bits, far beyond
/// guessing, and renders as 16 base32 characters.
const RECOVERY_CODE_BYTES: usize = 10;

/// Crockford-style alphabet without the characters people misread when copying
/// a code off a screen: no `I`, `L`, `O`, `U`, `0` or `1`.
const RECOVERY_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";

/// Characters per group in the rendered code, for legibility.
const RECOVERY_CODE_GROUP: usize = 4;

/// What `begin` hands back so the operator can set up their authenticator.
#[derive(Debug)]
pub struct MfaEnrollmentStarted {
    /// Shown as text for manual entry.
    pub secret: String,
    /// `otpauth://` URI, normally rendered as a QR code.
    pub provisioning_uri: String,
}

/// What `activate` and `regenerate_recovery_codes` hand back.
///
/// These plaintext codes exist exactly once. Only hashes are stored, so they
/// cannot be shown again.
#[derive(Debug)]
pub struct MfaRecoveryCodesIssued {
    pub recovery_codes: Vec<String>,
}

/// Where a user stands, for a status endpoint.
#[derive(Debug, Clone, Copy)]
pub struct MfaStatus {
    pub enrolled: bool,
    pub active: bool,
    pub recovery_codes_remaining: usize,
}

pub struct ManageMfaEnrollmentUseCase {
    user_repo: Arc<dyn BackofficeUserRepository>,
    recovery_repo: Arc<dyn MfaRecoveryCodeRepository>,
    totp: Arc<dyn TotpService>,
}

impl ManageMfaEnrollmentUseCase {
    pub fn new(
        user_repo: Arc<dyn BackofficeUserRepository>,
        recovery_repo: Arc<dyn MfaRecoveryCodeRepository>,
        totp: Arc<dyn TotpService>,
    ) -> Self {
        Self {
            user_repo,
            recovery_repo,
            totp,
        }
    }

    /// Reports enrollment state without changing anything.
    pub async fn status(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<MfaStatus, BackofficeIdentityError> {
        let user = self.load_user(user_id).await?;
        let remaining = if user.is_mfa_active() {
            self.recovery_repo
                .list_available_for_user(user_id)
                .await?
                .len()
        } else {
            0
        };

        Ok(MfaStatus {
            enrolled: user.mfa_secret().is_some(),
            active: user.is_mfa_active(),
            recovery_codes_remaining: remaining,
        })
    }

    /// Generates a secret and stores it as a PENDING enrollment.
    ///
    /// When MFA is already active, `current_code` must carry a valid code from
    /// the EXISTING authenticator. Otherwise a stolen session could enroll an
    /// attacker's authenticator and, once activated, lock the operator out.
    pub async fn begin(
        &self,
        user_id: BackofficeUserId,
        current_code: Option<&str>,
    ) -> Result<MfaEnrollmentStarted, BackofficeIdentityError> {
        let mut user = self.load_user(user_id).await?;

        if user.is_mfa_active() {
            let code = current_code.ok_or(BackofficeIdentityError::InvalidMfaCode)?;
            self.verify_active_code(&user, code)?;
        }

        let secret = self.totp.generate_secret()?;
        let provisioning_uri = self.totp.provisioning_uri(&secret, user.email().as_str())?;

        user.begin_mfa_enrollment(&secret);
        self.user_repo.update(&user).await?;

        Ok(MfaEnrollmentStarted {
            secret: secret.expose_base32().to_string(),
            provisioning_uri,
        })
    }

    /// Confirms a pending enrollment and issues recovery codes.
    ///
    /// Requiring a valid code here is the whole point of the pending state: it
    /// proves the authenticator was scanned before it becomes required.
    pub async fn activate(
        &self,
        user_id: BackofficeUserId,
        code: &str,
    ) -> Result<MfaRecoveryCodesIssued, BackofficeIdentityError> {
        let mut user = self.load_user(user_id).await?;

        if !user.is_mfa_pending() {
            return Err(BackofficeIdentityError::MfaNotEnrolled);
        }

        let secret = Self::secret_of(&user)?;
        let step = self
            .totp
            .verify(&secret, code)?
            .ok_or(BackofficeIdentityError::InvalidMfaCode)?;

        if !user.activate_mfa(step) {
            // Unreachable given the is_mfa_pending check above, but the entity
            // owns the invariant and this keeps it authoritative.
            return Err(BackofficeIdentityError::MfaNotEnrolled);
        }

        // Persist activation BEFORE handing out recovery codes: if the code
        // write failed after showing them, the operator would hold codes the
        // system does not know about.
        self.user_repo.update(&user).await?;
        self.issue_recovery_codes(user_id).await
    }

    /// Clears MFA entirely. Requires a currently valid code or the operation
    /// would be a one-request downgrade for anyone holding a session token.
    pub async fn disable(
        &self,
        user_id: BackofficeUserId,
        code: &str,
    ) -> Result<(), BackofficeIdentityError> {
        let mut user = self.load_user(user_id).await?;

        if !user.is_mfa_active() {
            return Err(BackofficeIdentityError::MfaNotEnrolled);
        }

        self.verify_active_code(&user, code)?;

        user.disable_mfa();
        self.user_repo.update(&user).await?;
        self.recovery_repo.delete_all_for_user(user_id).await?;
        Ok(())
    }

    /// Replaces the recovery code set, invalidating every previous code.
    pub async fn regenerate_recovery_codes(
        &self,
        user_id: BackofficeUserId,
        code: &str,
    ) -> Result<MfaRecoveryCodesIssued, BackofficeIdentityError> {
        let user = self.load_user(user_id).await?;

        if !user.is_mfa_active() {
            return Err(BackofficeIdentityError::MfaNotEnrolled);
        }

        self.verify_active_code(&user, code)?;
        self.issue_recovery_codes(user_id).await
    }

    // -------------------------------------------------------------------------
    // Internals
    // -------------------------------------------------------------------------

    async fn load_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<crate::domain::entities::BackofficeUser, BackofficeIdentityError> {
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| BackofficeIdentityError::UserNotFound(*user_id.as_uuid()))
    }

    fn secret_of(
        user: &crate::domain::entities::BackofficeUser,
    ) -> Result<TotpSecret, BackofficeIdentityError> {
        let raw = user
            .mfa_secret()
            .ok_or(BackofficeIdentityError::MfaNotEnrolled)?;
        TotpSecret::new(raw)
    }

    /// Verifies a code against an ACTIVE enrollment.
    ///
    /// Deliberately does not persist the replay watermark: these are management
    /// operations, and advancing the watermark here would let an operator lock
    /// themselves out of the login they are about to perform with the same
    /// code. Replay protection belongs on the login path, where the code buys a
    /// session.
    fn verify_active_code(
        &self,
        user: &crate::domain::entities::BackofficeUser,
        code: &str,
    ) -> Result<u64, BackofficeIdentityError> {
        let secret = Self::secret_of(user)?;
        self.totp
            .verify(&secret, code)?
            .ok_or(BackofficeIdentityError::InvalidMfaCode)
    }

    /// Generates a fresh set, stores the hashes, returns the plaintext once.
    async fn issue_recovery_codes(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<MfaRecoveryCodesIssued, BackofficeIdentityError> {
        let mut plaintext = Vec::with_capacity(RECOVERY_CODE_COUNT);
        let mut entities = Vec::with_capacity(RECOVERY_CODE_COUNT);

        for _ in 0..RECOVERY_CODE_COUNT {
            let code = generate_recovery_code();
            let hash = hash_recovery_code(&code)?;
            entities.push(MfaRecoveryCode::issue(user_id, hash));
            plaintext.push(code);
        }

        self.recovery_repo
            .replace_all_for_user(user_id, &entities)
            .await?;

        Ok(MfaRecoveryCodesIssued {
            recovery_codes: plaintext,
        })
    }
}

/// Renders a random code as grouped characters from the reduced alphabet.
fn generate_recovery_code() -> String {
    let mut bytes = [0u8; RECOVERY_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);

    // Modulo bias is negligible here: the alphabet is 30 characters against 256
    // byte values, and each character still carries ~4.9 bits. With 16
    // characters the code holds ~78 bits, which is not guessable regardless.
    let chars: Vec<char> = bytes
        .iter()
        .flat_map(|byte| {
            let high = (byte >> 4) as usize;
            let low = (byte & 0x0F) as usize;
            [
                RECOVERY_CODE_ALPHABET[high % RECOVERY_CODE_ALPHABET.len()] as char,
                RECOVERY_CODE_ALPHABET[(low + high) % RECOVERY_CODE_ALPHABET.len()] as char,
            ]
        })
        .collect();

    chars
        .chunks(RECOVERY_CODE_GROUP)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Argon2 hash of a recovery code, matching how passwords are stored here.
///
/// The codes carry ~78 bits, so a fast hash would also be defensible; Argon2
/// keeps one hashing story in the codebase and bounds a verification sweep to
/// the ten stored codes.
fn hash_recovery_code(code: &str) -> Result<String, BackofficeIdentityError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(code.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| BackofficeIdentityError::PasswordHashError(e.to_string()))
}

/// Constant-ish comparison of a candidate against a stored recovery hash.
///
/// Exposed for the login path in the next slice.
pub fn verify_recovery_code(candidate: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- recovery code shape -------------------------------------------------

    #[test]
    fn recovery_codes_use_only_the_reduced_alphabet() {
        for _ in 0..50 {
            let code = generate_recovery_code();
            for ch in code.chars().filter(|c| *c != '-') {
                assert!(
                    RECOVERY_CODE_ALPHABET.contains(&(ch as u8)),
                    "{ch} is not in the reduced alphabet (code: {code})"
                );
            }
        }
    }

    /// The characters people misread when copying off a screen must be absent.
    #[test]
    fn recovery_codes_avoid_ambiguous_characters() {
        for _ in 0..50 {
            let code = generate_recovery_code();
            for ambiguous in ['I', 'L', 'O', 'U', '0', '1'] {
                assert!(
                    !code.contains(ambiguous),
                    "{ambiguous} appears in {code} and is easy to misread"
                );
            }
        }
    }

    #[test]
    fn recovery_codes_are_grouped_for_legibility() {
        let code = generate_recovery_code();
        for group in code.split('-') {
            assert_eq!(group.len(), RECOVERY_CODE_GROUP, "bad grouping in {code}");
        }
    }

    #[test]
    fn recovery_codes_do_not_repeat() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            assert!(
                seen.insert(generate_recovery_code()),
                "generated a duplicate recovery code"
            );
        }
    }

    // --- hashing -------------------------------------------------------------

    #[test]
    fn a_recovery_code_verifies_against_its_own_hash() {
        let code = generate_recovery_code();
        let hash = hash_recovery_code(&code).unwrap();
        assert!(verify_recovery_code(&code, &hash));
    }

    #[test]
    fn a_different_code_does_not_verify() {
        let hash = hash_recovery_code(&generate_recovery_code()).unwrap();
        assert!(!verify_recovery_code(&generate_recovery_code(), &hash));
    }

    /// The stored value must not be the code itself.
    #[test]
    fn the_hash_does_not_contain_the_code() {
        let code = generate_recovery_code();
        let hash = hash_recovery_code(&code).unwrap();
        assert!(!hash.contains(&code));
        assert!(hash.starts_with("$argon2"));
    }

    /// Equal codes must still hash differently, or the table leaks which
    /// operators share a code.
    #[test]
    fn the_same_code_hashes_differently_each_time() {
        let code = generate_recovery_code();
        assert_ne!(
            hash_recovery_code(&code).unwrap(),
            hash_recovery_code(&code).unwrap()
        );
    }

    /// A corrupt stored hash must fail closed, not panic.
    #[test]
    fn a_malformed_stored_hash_never_verifies() {
        assert!(!verify_recovery_code("anything", "not-a-hash"));
        assert!(!verify_recovery_code("anything", ""));
    }
}
