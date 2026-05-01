use std::sync::Arc;

use crate::AccountingError;
use crate::application::dtos::CreateAccountCommand;
use crate::domain::entities::Account;
use crate::domain::repositories::AccountRepository;

pub struct CreateAccountUseCase {
    repo: Arc<dyn AccountRepository>,
}

impl CreateAccountUseCase {
    pub fn new(repo: Arc<dyn AccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, cmd: CreateAccountCommand) -> Result<Account, AccountingError> {
        if let Some(_existing) = self.repo.find_by_code(&cmd.code).await? {
            return Err(AccountingError::DuplicateAccountCode(cmd.code));
        }
        let account = Account::create(cmd.code, cmd.name, cmd.account_type, cmd.parent_id);
        self.repo.save(&account).await?;
        Ok(account)
    }
}
