use std::sync::Arc;

use crate::CashManagementError;
use crate::application::dtos::CreateBankAccountCommand;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;

pub struct CreateBankAccountUseCase {
    repo: Arc<dyn BankAccountRepository>,
}

impl CreateBankAccountUseCase {
    pub fn new(repo: Arc<dyn BankAccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateBankAccountCommand,
    ) -> Result<BankAccount, CashManagementError> {
        if let Some(_existing) = self
            .repo
            .find_by_account_number(&cmd.account_number)
            .await?
        {
            return Err(CashManagementError::DuplicateAccountNumber(
                cmd.account_number,
            ));
        }
        let account = BankAccount::create(
            cmd.store_id,
            cmd.bank_name,
            cmd.account_number,
            cmd.account_type,
            cmd.currency,
            cmd.opening_balance,
        )?;
        self.repo.save(&account).await?;
        Ok(account)
    }
}
