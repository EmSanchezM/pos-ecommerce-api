//! A single-use recovery code for a backoffice operator.
//!
//! Only the Argon2 hash is ever persisted. The plaintext exists exactly once —
//! in the response that issues it — and is unrecoverable afterwards, so a
//! leaked table yields nothing usable.

use chrono::{DateTime, Utc};
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::value_objects::BackofficeUserId;

/// A recovery code as stored: hashed, and marked once consumed.
#[derive(Debug, Clone)]
pub struct MfaRecoveryCode {
    id: Uuid,
    backoffice_user_id: BackofficeUserId,
    code_hash: String,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl MfaRecoveryCode {
    /// Rehydrates from storage.
    pub fn new(
        id: Uuid,
        backoffice_user_id: BackofficeUserId,
        code_hash: String,
        used_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            backoffice_user_id,
            code_hash,
            used_at,
            created_at,
        }
    }

    /// Creates an unused code from an already-hashed value.
    ///
    /// Takes the hash rather than the plaintext so this type never holds a
    /// usable code, and hashing stays where the hasher lives.
    pub fn issue(backoffice_user_id: BackofficeUserId, code_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            backoffice_user_id,
            code_hash,
            used_at: None,
            created_at: now,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn backoffice_user_id(&self) -> BackofficeUserId {
        self.backoffice_user_id
    }

    pub fn code_hash(&self) -> &str {
        &self.code_hash
    }

    pub fn used_at(&self) -> Option<DateTime<Utc>> {
        self.used_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// True while the code can still be redeemed.
    pub fn is_available(&self) -> bool {
        self.used_at.is_none()
    }

    /// Marks the code consumed. Idempotent: re-consuming keeps the original
    /// timestamp, so the audit trail shows when it was actually first used.
    pub fn consume(&mut self) {
        if self.used_at.is_none() {
            self.used_at = Some(Utc::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue() -> MfaRecoveryCode {
        MfaRecoveryCode::issue(BackofficeUserId::new(), "$argon2id$fake".to_string())
    }

    #[test]
    fn a_freshly_issued_code_is_available() {
        let code = issue();
        assert!(code.is_available());
        assert!(code.used_at().is_none());
    }

    #[test]
    fn consuming_marks_it_unavailable() {
        let mut code = issue();
        code.consume();
        assert!(!code.is_available());
        assert!(code.used_at().is_some());
    }

    /// A code must buy exactly one login. Re-consuming must not refresh the
    /// record of when it was first used.
    #[test]
    fn consuming_twice_keeps_the_first_timestamp() {
        let mut code = issue();
        code.consume();
        let first = code.used_at().unwrap();

        code.consume();
        assert_eq!(code.used_at().unwrap(), first);
        assert!(!code.is_available());
    }

    #[test]
    fn issued_codes_get_distinct_ids() {
        assert_ne!(issue().id(), issue().id());
    }

    /// The entity carries the hash only — never anything redeemable.
    #[test]
    fn stores_the_hash_it_was_given() {
        let code = MfaRecoveryCode::issue(BackofficeUserId::new(), "$argon2id$abc".to_string());
        assert_eq!(code.code_hash(), "$argon2id$abc");
    }
}
