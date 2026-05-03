use async_trait::async_trait;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyProgram;
use crate::domain::value_objects::LoyaltyProgramId;

#[async_trait]
pub trait LoyaltyProgramRepository: Send + Sync {
    async fn save(&self, program: &LoyaltyProgram) -> Result<(), LoyaltyError>;
    async fn update(&self, program: &LoyaltyProgram) -> Result<(), LoyaltyError>;
    async fn find_by_id(
        &self,
        id: LoyaltyProgramId,
    ) -> Result<Option<LoyaltyProgram>, LoyaltyError>;
    async fn list(&self, store_id: Option<Uuid>) -> Result<Vec<LoyaltyProgram>, LoyaltyError>;
}
