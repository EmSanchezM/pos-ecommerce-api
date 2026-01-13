// User entity - represents a system user with authentication and profile data

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{Email, UserId, Username};

/// User entity representing a person with access to the system
///
/// Contains authentication credentials, profile information, and status.
/// Users can be assigned to multiple stores and have different roles per store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    first_name: String,
    last_name: String,
    password_hash: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new User with the provided data
    ///
    /// The password should already be hashed before calling this constructor.
    /// Use `UserBuilder` for a more ergonomic construction experience.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UserId,
        username: Username,
        email: Email,
        first_name: String,
        last_name: String,
        password_hash: String,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            first_name,
            last_name,
            password_hash,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Creates a new active User with current timestamps
    pub fn create(
        username: Username,
        email: Email,
        first_name: String,
        last_name: String,
        password_hash: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            username,
            email,
            first_name,
            last_name,
            password_hash,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns the user's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    // Setters / Mutators

    /// Updates the user's first name
    pub fn set_first_name(&mut self, first_name: String) {
        self.first_name = first_name;
        self.updated_at = Utc::now();
    }

    /// Updates the user's last name
    pub fn set_last_name(&mut self, last_name: String) {
        self.last_name = last_name;
        self.updated_at = Utc::now();
    }

    /// Updates the user's email
    pub fn set_email(&mut self, email: Email) {
        self.email = email;
        self.updated_at = Utc::now();
    }

    /// Updates the user's password hash
    pub fn set_password_hash(&mut self, password_hash: String) {
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
    }

    /// Activates the user account
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates the user account
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for User {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hashed_password".to_string(),
        )
    }

    #[test]
    fn test_user_create() {
        let user = create_test_user();

        assert_eq!(user.username().as_str(), "testuser");
        assert_eq!(user.email().as_str(), "test@example.com");
        assert_eq!(user.first_name(), "John");
        assert_eq!(user.last_name(), "Doe");
        assert_eq!(user.password_hash(), "hashed_password");
        assert!(user.is_active());
    }

    #[test]
    fn test_user_full_name() {
        let user = create_test_user();
        assert_eq!(user.full_name(), "John Doe");
    }

    #[test]
    fn test_user_set_first_name() {
        let mut user = create_test_user();
        let original_updated = user.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));
        user.set_first_name("Jane".to_string());

        assert_eq!(user.first_name(), "Jane");
        assert!(user.updated_at() > original_updated);
    }

    #[test]
    fn test_user_set_last_name() {
        let mut user = create_test_user();
        user.set_last_name("Smith".to_string());
        assert_eq!(user.last_name(), "Smith");
    }

    #[test]
    fn test_user_set_email() {
        let mut user = create_test_user();
        let new_email = Email::new("new@example.com").unwrap();
        user.set_email(new_email);
        assert_eq!(user.email().as_str(), "new@example.com");
    }

    #[test]
    fn test_user_activate_deactivate() {
        let mut user = create_test_user();
        assert!(user.is_active());

        user.deactivate();
        assert!(!user.is_active());

        user.activate();
        assert!(user.is_active());
    }

    #[test]
    fn test_user_equality_by_id() {
        let user1 = create_test_user();
        let user2 = User::new(
            *user1.id(),
            Username::new("different").unwrap(),
            Email::new("different@example.com").unwrap(),
            "Different".to_string(),
            "Name".to_string(),
            "different_hash".to_string(),
            false,
            Utc::now(),
            Utc::now(),
        );

        // Users are equal if they have the same ID
        assert_eq!(user1, user2);
    }

    #[test]
    fn test_user_inequality_different_ids() {
        let user1 = create_test_user();
        let user2 = create_test_user();

        // Different IDs mean different users
        assert_ne!(user1, user2);
    }
}
