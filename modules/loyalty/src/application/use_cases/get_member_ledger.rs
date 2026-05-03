use std::sync::Arc;

use crate::LoyaltyError;
use crate::domain::entities::PointsLedgerEntry;
use crate::domain::repositories::PointsLedgerRepository;
use crate::domain::value_objects::LoyaltyMemberId;

pub struct GetMemberLedgerUseCase {
    repo: Arc<dyn PointsLedgerRepository>,
}

impl GetMemberLedgerUseCase {
    pub fn new(repo: Arc<dyn PointsLedgerRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        member_id: LoyaltyMemberId,
        limit: i64,
    ) -> Result<Vec<PointsLedgerEntry>, LoyaltyError> {
        let limit = limit.clamp(1, 500);
        self.repo.list_for_member(member_id, limit).await
    }
}
