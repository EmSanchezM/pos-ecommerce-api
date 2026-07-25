use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{BackofficeEmail, BackofficeUserId, TotpSecret};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficeUser {
    id: BackofficeUserId,
    email: BackofficeEmail,
    password_hash: String,
    mfa_secret: Option<String>,
    /// Set when the operator proved they can produce a valid code. A secret
    /// without this is a PENDING enrollment and must never gate a login.
    mfa_activated_at: Option<DateTime<Utc>>,
    /// Highest TOTP step already accepted, for replay rejection.
    mfa_last_used_step: Option<u64>,
    is_active: bool,
    last_login_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BackofficeUser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BackofficeUserId,
        email: BackofficeEmail,
        password_hash: String,
        mfa_secret: Option<String>,
        is_active: bool,
        last_login_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            mfa_secret,
            mfa_activated_at: None,
            mfa_last_used_step: None,
            is_active,
            last_login_at,
            created_at,
            updated_at,
        }
    }

    /// Rehydrates a row that carries MFA state.
    ///
    /// Kept separate from [`Self::new`] so the many existing call sites that
    /// predate MFA keep compiling unchanged; persistence uses this one.
    #[allow(clippy::too_many_arguments)]
    pub fn with_mfa_state(
        mut self,
        mfa_activated_at: Option<DateTime<Utc>>,
        mfa_last_used_step: Option<u64>,
    ) -> Self {
        self.mfa_activated_at = mfa_activated_at;
        self.mfa_last_used_step = mfa_last_used_step;
        self
    }

    pub fn create(email: BackofficeEmail, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: BackofficeUserId::new(),
            email,
            password_hash,
            mfa_secret: None,
            mfa_activated_at: None,
            mfa_last_used_step: None,
            is_active: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> &BackofficeUserId {
        &self.id
    }

    pub fn email(&self) -> &BackofficeEmail {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn mfa_secret(&self) -> Option<&str> {
        self.mfa_secret.as_deref()
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // MFA enrollment state machine
    //
    //   none --begin_mfa_enrollment--> pending --activate_mfa--> active
    //     ^                               |                        |
    //     +---------disable_mfa-----------+------------------------+
    //
    // The pending state is the point of the whole design: a secret is generated
    // before the operator has scanned it, and treating that as protection would
    // lock them out of a platform-owner account the moment a scan failed.
    // =========================================================================

    pub fn mfa_activated_at(&self) -> Option<DateTime<Utc>> {
        self.mfa_activated_at
    }

    pub fn mfa_last_used_step(&self) -> Option<u64> {
        self.mfa_last_used_step
    }

    /// True only once enrollment has been confirmed. This is the ONLY predicate
    /// login should consult — `mfa_secret().is_some()` is not the same question.
    pub fn is_mfa_active(&self) -> bool {
        self.mfa_secret.is_some() && self.mfa_activated_at.is_some()
    }

    /// True while a secret exists that has not yet been confirmed.
    pub fn is_mfa_pending(&self) -> bool {
        self.mfa_secret.is_some() && self.mfa_activated_at.is_none()
    }

    /// Stores a freshly generated secret as a PENDING enrollment.
    ///
    /// Re-enrolling while pending is allowed and replaces the secret — an
    /// operator whose first scan failed should be able to start over. Re-
    /// enrolling while ACTIVE also replaces it and drops back to pending, so
    /// the old authenticator stops working only once the new one is confirmed.
    pub fn begin_mfa_enrollment(&mut self, secret: &TotpSecret) {
        self.mfa_secret = Some(secret.expose_base32().to_string());
        self.mfa_activated_at = None;
        self.mfa_last_used_step = None;
        self.updated_at = Utc::now();
    }

    /// Confirms a pending enrollment, recording the step that proved it.
    ///
    /// Returns false when there is nothing pending, so a caller cannot activate
    /// MFA for a user who never enrolled.
    pub fn activate_mfa(&mut self, verified_step: u64) -> bool {
        if !self.is_mfa_pending() {
            return false;
        }
        let now = Utc::now();
        self.mfa_activated_at = Some(now);
        self.mfa_last_used_step = Some(verified_step);
        self.updated_at = now;
        true
    }

    /// Clears MFA entirely, returning the user to password-only.
    pub fn disable_mfa(&mut self) {
        self.mfa_secret = None;
        self.mfa_activated_at = None;
        self.mfa_last_used_step = None;
        self.updated_at = Utc::now();
    }

    /// True when `step` has already been used, i.e. this code is a replay.
    ///
    /// A TOTP code stays valid for its whole window, so without this a code
    /// observed in transit can be presented again until the window closes.
    pub fn is_replayed_step(&self, step: u64) -> bool {
        matches!(self.mfa_last_used_step, Some(last) if step <= last)
    }

    /// Records a successfully used step. Never moves backwards, so an
    /// out-of-order acceptance cannot reopen an already-consumed window.
    pub fn record_mfa_step(&mut self, step: u64) {
        if self.mfa_last_used_step.is_none_or(|last| step > last) {
            self.mfa_last_used_step = Some(step);
            self.updated_at = Utc::now();
        }
    }
}

impl PartialEq for BackofficeUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BackofficeUser {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_email() -> BackofficeEmail {
        BackofficeEmail::new("admin@example.com").unwrap()
    }

    #[test]
    fn test_create_defaults_is_active_true() {
        let user = BackofficeUser::create(make_email(), "hash".to_string());
        assert!(user.is_active());
    }

    #[test]
    fn test_create_mfa_secret_starts_none() {
        let user = BackofficeUser::create(make_email(), "hash".to_string());
        assert!(user.mfa_secret().is_none());
    }

    #[test]
    fn test_create_last_login_at_starts_none() {
        let user = BackofficeUser::create(make_email(), "hash".to_string());
        assert!(user.last_login_at().is_none());
    }

    #[test]
    fn test_deactivate_and_activate() {
        let mut user = BackofficeUser::create(make_email(), "hash".to_string());
        user.deactivate();
        assert!(!user.is_active());
        user.activate();
        assert!(user.is_active());
    }

    #[test]
    fn test_record_login_sets_last_login_at() {
        let mut user = BackofficeUser::create(make_email(), "hash".to_string());
        assert!(user.last_login_at().is_none());
        user.record_login();
        assert!(user.last_login_at().is_some());
    }

    #[test]
    fn test_equality_by_id() {
        let user1 = BackofficeUser::create(make_email(), "hash".to_string());
        let user2 = BackofficeUser::new(
            *user1.id(),
            BackofficeEmail::new("other@example.com").unwrap(),
            "other_hash".to_string(),
            None,
            false,
            None,
            Utc::now(),
            Utc::now(),
        );
        assert_eq!(user1, user2);
    }

    #[test]
    fn test_inequality_different_ids() {
        let user1 = BackofficeUser::create(make_email(), "hash".to_string());
        let user2 = BackofficeUser::create(make_email(), "hash".to_string());
        assert_ne!(user1, user2);
    }

    // =========================================================================
    // MFA enrollment state machine
    // =========================================================================

    fn make_secret() -> TotpSecret {
        TotpSecret::new("JBSWY3DPEHPK3PXPJBSWY3DPEH").unwrap()
    }

    fn make_user() -> BackofficeUser {
        BackofficeUser::create(make_email(), "hash".to_string())
    }

    #[test]
    fn mfa_starts_neither_active_nor_pending() {
        let user = make_user();
        assert!(!user.is_mfa_active());
        assert!(!user.is_mfa_pending());
        assert!(user.mfa_activated_at().is_none());
    }

    /// THE invariant: a generated-but-unconfirmed secret is not protection.
    /// Treating it as active would lock an operator out of a platform-owner
    /// account the moment their authenticator failed to scan.
    #[test]
    fn beginning_enrollment_is_pending_not_active() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());

        assert!(user.is_mfa_pending());
        assert!(
            !user.is_mfa_active(),
            "a pending enrollment must NEVER count as active MFA"
        );
        assert!(user.mfa_secret().is_some());
    }

    #[test]
    fn activating_a_pending_enrollment_makes_it_active() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());

        assert!(user.activate_mfa(42));
        assert!(user.is_mfa_active());
        assert!(!user.is_mfa_pending());
        assert_eq!(user.mfa_last_used_step(), Some(42));
        assert!(user.mfa_activated_at().is_some());
    }

    /// Activation must be impossible for a user who never enrolled — otherwise
    /// a caller could mark MFA active with no secret behind it.
    #[test]
    fn activating_without_enrolling_is_refused() {
        let mut user = make_user();
        assert!(!user.activate_mfa(42));
        assert!(!user.is_mfa_active());
    }

    #[test]
    fn activating_twice_is_refused() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        assert!(user.activate_mfa(42));

        assert!(
            !user.activate_mfa(99),
            "an already-active enrollment cannot be re-activated"
        );
        assert_eq!(user.mfa_last_used_step(), Some(42));
    }

    /// A failed scan must be recoverable by starting over.
    #[test]
    fn re_enrolling_while_pending_replaces_the_secret() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        let first = user.mfa_secret().unwrap().to_string();

        let other = TotpSecret::new("KRSXG5DJNZTXIZLSKRSXG5DJNZ").unwrap();
        user.begin_mfa_enrollment(&other);

        assert_ne!(user.mfa_secret().unwrap(), first);
        assert!(user.is_mfa_pending());
    }

    /// Re-enrolling from active drops back to pending: the new authenticator
    /// must be proven before the old one stops working.
    #[test]
    fn re_enrolling_while_active_drops_back_to_pending() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(10);

        user.begin_mfa_enrollment(&make_secret());
        assert!(user.is_mfa_pending());
        assert!(!user.is_mfa_active());
        assert_eq!(
            user.mfa_last_used_step(),
            None,
            "a new secret must not inherit the old replay watermark"
        );
    }

    #[test]
    fn disabling_clears_every_trace_of_mfa() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(10);

        user.disable_mfa();
        assert!(!user.is_mfa_active());
        assert!(!user.is_mfa_pending());
        assert!(user.mfa_secret().is_none());
        assert!(user.mfa_activated_at().is_none());
        assert!(user.mfa_last_used_step().is_none());
    }

    // --- replay rejection ----------------------------------------------------

    /// A code stays valid for its whole window; presenting it twice must fail.
    #[test]
    fn the_step_that_was_just_used_is_a_replay() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(100);

        assert!(user.is_replayed_step(100), "the same step must be rejected");
    }

    #[test]
    fn an_older_step_is_a_replay() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(100);

        assert!(user.is_replayed_step(99));
    }

    #[test]
    fn a_newer_step_is_accepted() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(100);

        assert!(!user.is_replayed_step(101));
    }

    /// With no watermark yet, nothing can be a replay.
    #[test]
    fn no_recorded_step_means_nothing_is_replayed() {
        let user = make_user();
        assert!(!user.is_replayed_step(0));
        assert!(!user.is_replayed_step(u64::MAX));
    }

    #[test]
    fn recording_advances_the_watermark() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(100);

        user.record_mfa_step(105);
        assert_eq!(user.mfa_last_used_step(), Some(105));
    }

    /// The watermark must never move backwards, or an already-consumed window
    /// would become usable again.
    #[test]
    fn recording_an_older_step_does_not_move_the_watermark_back() {
        let mut user = make_user();
        user.begin_mfa_enrollment(&make_secret());
        user.activate_mfa(100);

        user.record_mfa_step(50);
        assert_eq!(user.mfa_last_used_step(), Some(100));
        assert!(user.is_replayed_step(50));
    }

    #[test]
    fn recording_the_first_step_sets_the_watermark() {
        let mut user = make_user();
        user.record_mfa_step(7);
        assert_eq!(user.mfa_last_used_step(), Some(7));
    }
}
