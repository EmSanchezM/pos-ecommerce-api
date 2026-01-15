// ApproveAdjustmentUseCase - approves or rejects a pending adjustment

use std::sync::Arc;

use crate::application::dtos::commands::ApproveAdjustmentCommand;
use crate::application::dtos::responses::{AdjustmentDetailResponse, AdjustmentItemResponse};
use crate::domain::entities::StockAdjustment;
use crate::domain::repositories::AdjustmentRepository;
use crate::domain::value_objects::AdjustmentId;
use crate::InventoryError;
use identity::UserId;

/// Use case for approving or rejecting a stock adjustment.
///
/// Validates status is pending_approval, records approver and timestamp,
/// and changes status to approved or rejected.
pub struct ApproveAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    adjustment_repo: Arc<A>,
}

impl<A> ApproveAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    /// Creates a new instance of ApproveAdjustmentUseCase
    pub fn new(adjustment_repo: Arc<A>) -> Self {
        Self { adjustment_repo }
    }

    /// Executes the use case to approve or reject an adjustment
    ///
    /// # Arguments
    /// * `command` - The approve adjustment command containing adjustment ID and decision
    /// * `approver_id` - ID of the user approving/rejecting the adjustment
    ///
    /// # Returns
    /// AdjustmentDetailResponse on success
    ///
    /// # Errors
    /// * `InventoryError::AdjustmentNotFound` - If adjustment doesn't exist
    /// * `InventoryError::InvalidStatusTransition` - If adjustment is not in pending_approval status
    pub async fn execute(
        &self,
        command: ApproveAdjustmentCommand,
        approver_id: UserId,
    ) -> Result<AdjustmentDetailResponse, InventoryError> {
        // 1. Find adjustment with items
        let adjustment_id = AdjustmentId::from_uuid(command.adjustment_id);
        let mut adjustment = self
            .adjustment_repo
            .find_by_id_with_items(adjustment_id)
            .await?
            .ok_or(InventoryError::AdjustmentNotFound(command.adjustment_id))?;

        // 2. Approve or reject based on command
        if command.approve {
            adjustment.approve(approver_id)?;
        } else {
            adjustment.reject(approver_id)?;
        }

        // 3. Update notes if provided
        // Note: We can't update notes after submission, so we skip this
        // The notes in the command are for audit purposes only

        // 4. Update adjustment
        self.adjustment_repo.update(&adjustment).await?;

        // 5. Convert to response
        Ok(self.to_response(&adjustment))
    }

    fn to_response(&self, adjustment: &StockAdjustment) -> AdjustmentDetailResponse {
        let items: Vec<AdjustmentItemResponse> = adjustment
            .items()
            .iter()
            .map(|item| AdjustmentItemResponse {
                id: item.id(),
                adjustment_id: item.adjustment_id().into_uuid(),
                stock_id: item.stock_id().into_uuid(),
                stock: None,
                quantity: item.quantity(),
                unit_cost: item.unit_cost(),
                balance_before: item.balance_before(),
                balance_after: item.balance_after(),
                notes: item.notes().map(|s| s.to_string()),
                created_at: item.created_at(),
            })
            .collect();

        AdjustmentDetailResponse {
            id: adjustment.id().into_uuid(),
            store_id: adjustment.store_id().into_uuid(),
            adjustment_number: adjustment.adjustment_number().to_string(),
            adjustment_type: adjustment.adjustment_type().to_string(),
            adjustment_reason: adjustment.adjustment_reason().to_string(),
            status: adjustment.status().to_string(),
            created_by_id: adjustment.created_by_id().into_uuid(),
            approved_by_id: adjustment.approved_by_id().map(|id| id.into_uuid()),
            approved_at: adjustment.approved_at(),
            applied_at: adjustment.applied_at(),
            notes: adjustment.notes().map(|s| s.to_string()),
            attachments: Some(adjustment.attachments().clone()),
            items,
            created_at: adjustment.created_at(),
            updated_at: adjustment.updated_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::AdjustmentItem;
    use crate::domain::value_objects::{AdjustmentReason, AdjustmentStatus, AdjustmentType, StockId};
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    struct MockAdjustmentRepository {
        adjustments: Mutex<HashMap<AdjustmentId, StockAdjustment>>,
    }

    impl MockAdjustmentRepository {
        fn new() -> Self {
            Self {
                adjustments: Mutex::new(HashMap::new()),
            }
        }

        fn add_adjustment(&self, adjustment: StockAdjustment) {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment);
        }
    }

    #[async_trait]
    impl AdjustmentRepository for MockAdjustmentRepository {
        async fn save(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: AdjustmentId,
        ) -> Result<Option<StockAdjustment>, InventoryError> {
            let adjustments = self.adjustments.lock().unwrap();
            Ok(adjustments.get(&id).cloned())
        }

        async fn find_by_id_with_items(
            &self,
            id: AdjustmentId,
        ) -> Result<Option<StockAdjustment>, InventoryError> {
            self.find_by_id(id).await
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<StockAdjustment>, InventoryError> {
            Ok(vec![])
        }

        async fn update(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment.clone());
            Ok(())
        }

        async fn generate_adjustment_number(
            &self,
            _store_id: StoreId,
        ) -> Result<String, InventoryError> {
            Ok("ADJ-TEST-00001".to_string())
        }
    }

    fn create_pending_adjustment() -> StockAdjustment {
        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            UserId::new(),
        );
        let item = AdjustmentItem::create(
            adjustment.id(),
            StockId::new(),
            dec!(-10),
            Some(dec!(5.00)),
        );
        adjustment.add_item(item).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment
    }

    #[tokio::test]
    async fn test_approve_adjustment_success() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let adjustment = create_pending_adjustment();
        let adjustment_id = adjustment.id();
        repo.add_adjustment(adjustment);

        let use_case = ApproveAdjustmentUseCase::new(repo.clone());
        let approver_id = UserId::new();

        let command = ApproveAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
            approve: true,
            notes: Some("Approved".to_string()),
        };

        let result = use_case.execute(command, approver_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "approved");
        assert!(response.approved_by_id.is_some());
        assert!(response.approved_at.is_some());

        // Verify in repository
        let updated = repo.find_by_id(adjustment_id).await.unwrap().unwrap();
        assert_eq!(updated.status(), AdjustmentStatus::Approved);
        assert_eq!(updated.approved_by_id(), Some(approver_id));
    }

    #[tokio::test]
    async fn test_reject_adjustment_success() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let adjustment = create_pending_adjustment();
        let adjustment_id = adjustment.id();
        repo.add_adjustment(adjustment);

        let use_case = ApproveAdjustmentUseCase::new(repo.clone());
        let approver_id = UserId::new();

        let command = ApproveAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
            approve: false,
            notes: Some("Rejected - insufficient documentation".to_string()),
        };

        let result = use_case.execute(command, approver_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "rejected");
        assert!(response.approved_by_id.is_some());
        assert!(response.approved_at.is_some());

        // Verify in repository
        let updated = repo.find_by_id(adjustment_id).await.unwrap().unwrap();
        assert_eq!(updated.status(), AdjustmentStatus::Rejected);
    }

    #[tokio::test]
    async fn test_approve_adjustment_not_found() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = ApproveAdjustmentUseCase::new(repo);

        let command = ApproveAdjustmentCommand {
            adjustment_id: new_uuid(),
            approve: true,
            notes: None,
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::AdjustmentNotFound(_))));
    }

    #[tokio::test]
    async fn test_approve_adjustment_wrong_status_draft() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        // Create draft adjustment (not submitted)
        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            UserId::new(),
        );
        let item = AdjustmentItem::create(
            adjustment.id(),
            StockId::new(),
            dec!(-10),
            None,
        );
        adjustment.add_item(item).unwrap();
        let adjustment_id = adjustment.id();
        repo.add_adjustment(adjustment);

        let use_case = ApproveAdjustmentUseCase::new(repo);

        let command = ApproveAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
            approve: true,
            notes: None,
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidStatusTransition)
        ));
    }

    #[tokio::test]
    async fn test_approve_adjustment_wrong_status_already_approved() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let mut adjustment = create_pending_adjustment();
        adjustment.approve(UserId::new()).unwrap(); // Already approved
        let adjustment_id = adjustment.id();
        repo.add_adjustment(adjustment);

        let use_case = ApproveAdjustmentUseCase::new(repo);

        let command = ApproveAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
            approve: true,
            notes: None,
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidStatusTransition)
        ));
    }
}
