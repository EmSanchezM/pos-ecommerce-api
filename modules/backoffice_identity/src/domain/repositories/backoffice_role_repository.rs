use async_trait::async_trait;

use crate::domain::entities::BackofficeRole;
use crate::domain::value_objects::BackofficeRoleId;
use crate::error::BackofficeIdentityError;

#[async_trait]
pub trait BackofficeRoleRepository: Send + Sync {
    async fn save(&self, role: &BackofficeRole) -> Result<(), BackofficeIdentityError>;

    async fn find_by_id(
        &self,
        id: BackofficeRoleId,
    ) -> Result<Option<BackofficeRole>, BackofficeIdentityError>;

    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<BackofficeRole>, BackofficeIdentityError>;

    async fn list(&self) -> Result<Vec<BackofficeRole>, BackofficeIdentityError>;
}
