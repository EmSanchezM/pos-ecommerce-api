use async_trait::async_trait;

use crate::domain::entities::BackofficePermission;
use crate::domain::value_objects::{BackofficePermissionId, PlatformPermissionCode};
use crate::error::BackofficeIdentityError;

#[async_trait]
pub trait BackofficePermissionRepository: Send + Sync {
    async fn save(&self, permission: &BackofficePermission) -> Result<(), BackofficeIdentityError>;

    async fn find_by_id(
        &self,
        id: BackofficePermissionId,
    ) -> Result<Option<BackofficePermission>, BackofficeIdentityError>;

    async fn find_by_code(
        &self,
        code: &PlatformPermissionCode,
    ) -> Result<Option<BackofficePermission>, BackofficeIdentityError>;

    async fn list(&self) -> Result<Vec<BackofficePermission>, BackofficeIdentityError>;
}
