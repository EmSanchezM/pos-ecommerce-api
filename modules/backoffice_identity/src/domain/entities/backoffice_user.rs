use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{BackofficeEmail, BackofficeUserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficeUser {
    id: BackofficeUserId,
    email: BackofficeEmail,
    password_hash: String,
    mfa_secret: Option<String>,
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
            is_active,
            last_login_at,
            created_at,
            updated_at,
        }
    }

    pub fn create(email: BackofficeEmail, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: BackofficeUserId::new(),
            email,
            password_hash,
            mfa_secret: None,
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
}
