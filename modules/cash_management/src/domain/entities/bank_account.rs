//! BankAccount — one row per (store, bank, account_number). `current_balance`
//! is the *book* balance (what we believe the bank holds based on entries we
//! recorded), not the live bank balance. `version` enables optimistic locking
//! since multiple writers may post transactions concurrently.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::value_objects::{BankAccountId, BankAccountType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    id: BankAccountId,
    store_id: Uuid,
    bank_name: String,
    account_number: String,
    account_type: BankAccountType,
    currency: String,
    current_balance: Decimal,
    is_active: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BankAccount {
    pub fn create(
        store_id: Uuid,
        bank_name: impl Into<String>,
        account_number: impl Into<String>,
        account_type: BankAccountType,
        currency: impl Into<String>,
        opening_balance: Decimal,
    ) -> Result<Self, CashManagementError> {
        if opening_balance < Decimal::ZERO {
            return Err(CashManagementError::NegativeAmount);
        }
        let now = Utc::now();
        Ok(Self {
            id: BankAccountId::new(),
            store_id,
            bank_name: bank_name.into(),
            account_number: account_number.into(),
            account_type,
            currency: currency.into(),
            current_balance: opening_balance,
            is_active: true,
            version: 0,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: BankAccountId,
        store_id: Uuid,
        bank_name: String,
        account_number: String,
        account_type: BankAccountType,
        currency: String,
        current_balance: Decimal,
        is_active: bool,
        version: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            bank_name,
            account_number,
            account_type,
            currency,
            current_balance,
            is_active,
            version,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, bank_name: impl Into<String>) {
        self.bank_name = bank_name.into();
        self.updated_at = Utc::now();
    }

    pub fn change_type(&mut self, account_type: BankAccountType) {
        self.account_type = account_type;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Apply a balance delta from a posted transaction. The caller passes the
    /// signed amount (positive for inflow, negative for outflow).
    pub fn apply_delta(&mut self, delta: Decimal) {
        self.current_balance += delta;
        self.updated_at = Utc::now();
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn id(&self) -> BankAccountId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn bank_name(&self) -> &str {
        &self.bank_name
    }
    pub fn account_number(&self) -> &str {
        &self.account_number
    }
    pub fn account_type(&self) -> BankAccountType {
        self.account_type
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn current_balance(&self) -> Decimal {
        self.current_balance
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn version(&self) -> i32 {
        self.version
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
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn fresh() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[test]
    fn create_starts_active_at_version_zero() {
        let a = BankAccount::create(
            fresh(),
            "BAC",
            "10-2345-6",
            BankAccountType::Checking,
            "HNL",
            dec!(1000),
        )
        .unwrap();
        assert!(a.is_active());
        assert_eq!(a.version(), 0);
        assert_eq!(a.current_balance(), dec!(1000));
    }

    #[test]
    fn create_rejects_negative_opening_balance() {
        let err = BankAccount::create(
            fresh(),
            "BAC",
            "10-2345-6",
            BankAccountType::Checking,
            "HNL",
            dec!(-1),
        )
        .unwrap_err();
        assert!(matches!(err, CashManagementError::NegativeAmount));
    }

    #[test]
    fn apply_delta_updates_balance_and_timestamp() {
        let mut a = BankAccount::create(
            fresh(),
            "BAC",
            "1",
            BankAccountType::Checking,
            "HNL",
            dec!(500),
        )
        .unwrap();
        let before = a.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(2));
        a.apply_delta(dec!(250));
        assert_eq!(a.current_balance(), dec!(750));
        assert!(a.updated_at() > before);
        a.apply_delta(dec!(-100));
        assert_eq!(a.current_balance(), dec!(650));
    }
}
