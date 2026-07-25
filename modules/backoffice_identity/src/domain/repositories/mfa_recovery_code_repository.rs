//! Storage port for single-use MFA recovery codes.

use async_trait::async_trait;

use crate::domain::entities::MfaRecoveryCode;
use crate::domain::value_objects::BackofficeUserId;
use crate::error::BackofficeIdentityError;

#[async_trait]
pub trait MfaRecoveryCodeRepository: Send + Sync {
    /// Replaces a user's entire code set in one transaction.
    ///
    /// Replace rather than append: issuing a new set must invalidate the old
    /// one, or a code printed a year ago still works. Atomic because a partial
    /// write would leave the operator holding codes that do not match what was
    /// stored.
    async fn replace_all_for_user(
        &self,
        user_id: BackofficeUserId,
        codes: &[MfaRecoveryCode],
    ) -> Result<(), BackofficeIdentityError>;

    /// Returns the user's codes that have not been consumed.
    async fn list_available_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<MfaRecoveryCode>, BackofficeIdentityError>;

    /// Marks one code consumed, and reports whether it actually was.
    ///
    /// Returns `false` when the row was already used, which makes the
    /// single-use guarantee a property of the DB rather than of a check-then-
    /// write the caller could race against.
    async fn consume(&self, code_id: uuid::Uuid) -> Result<bool, BackofficeIdentityError>;

    /// Deletes every code for a user, e.g. when MFA is disabled.
    async fn delete_all_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<(), BackofficeIdentityError>;
}
