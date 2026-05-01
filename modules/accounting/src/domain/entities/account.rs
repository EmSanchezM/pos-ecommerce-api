//! Account entity — one node in the chart of accounts. Codes are unique and
//! short (e.g. "1010" for Cash). Hierarchies are modeled via `parent_id`; the
//! domain doesn't enforce a depth limit but reports walk the tree to roll up
//! children into their parent for higher-level views.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{AccountId, AccountType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    id: AccountId,
    code: String,
    name: String,
    account_type: AccountType,
    parent_id: Option<AccountId>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Account {
    pub fn create(
        code: impl Into<String>,
        name: impl Into<String>,
        account_type: AccountType,
        parent_id: Option<AccountId>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: AccountId::new(),
            code: code.into(),
            name: name.into(),
            account_type,
            parent_id,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AccountId,
        code: String,
        name: String,
        account_type: AccountType,
        parent_id: Option<AccountId>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            name,
            account_type,
            parent_id,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> AccountId {
        self.id
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn account_type(&self) -> AccountType {
        self.account_type
    }
    pub fn parent_id(&self) -> Option<AccountId> {
        self.parent_id
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_assigns_active_status() {
        let a = Account::create("1010", "Cash", AccountType::Asset, None);
        assert!(a.is_active());
        assert_eq!(a.code(), "1010");
        assert_eq!(a.account_type(), AccountType::Asset);
    }

    #[test]
    fn deactivate_bumps_updated_at() {
        let mut a = Account::create("1010", "Cash", AccountType::Asset, None);
        let before = a.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(2));
        a.deactivate();
        assert!(!a.is_active());
        assert!(a.updated_at() > before);
    }
}
