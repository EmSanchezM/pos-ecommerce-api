use async_trait::async_trait;

use crate::domain::entities::{BackofficePermission, BackofficeRole, BackofficeUser};
use crate::domain::value_objects::{BackofficeEmail, BackofficeUserId};
use crate::error::BackofficeIdentityError;

#[async_trait]
pub trait BackofficeUserRepository: Send + Sync {
    async fn save(&self, user: &BackofficeUser) -> Result<(), BackofficeIdentityError>;

    async fn find_by_id(
        &self,
        id: BackofficeUserId,
    ) -> Result<Option<BackofficeUser>, BackofficeIdentityError>;

    async fn find_by_email(
        &self,
        email: &BackofficeEmail,
    ) -> Result<Option<BackofficeUser>, BackofficeIdentityError>;

    async fn update(&self, user: &BackofficeUser) -> Result<(), BackofficeIdentityError>;

    async fn list(&self) -> Result<Vec<BackofficeUser>, BackofficeIdentityError>;

    async fn list_roles_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<BackofficeRole>, BackofficeIdentityError>;

    async fn list_permissions_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<BackofficePermission>, BackofficeIdentityError>;

    async fn assign_role(
        &self,
        user_id: BackofficeUserId,
        role_id: crate::domain::value_objects::BackofficeRoleId,
    ) -> Result<(), BackofficeIdentityError>;

    async fn remove_role(
        &self,
        user_id: BackofficeUserId,
        role_id: crate::domain::value_objects::BackofficeRoleId,
    ) -> Result<(), BackofficeIdentityError>;
}
