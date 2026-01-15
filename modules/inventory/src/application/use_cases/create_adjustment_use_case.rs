// CreateAdjustmentUseCase - creates a new stock adjustment in draft status

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::commands::{AdjustmentItemCommand, CreateAdjustmentCommand};
use crate::application::dtos::responses::{AdjustmentDetailResponse, AdjustmentItemResponse};
use crate::domain::entities::{AdjustmentItem, StockAdjustment};
use crate::domain::repositories::AdjustmentRepository;
use crate::domain::value_objects::{AdjustmentReason, AdjustmentType, StockId};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// Use case for creating a new stock adjustment.
///
/// Generates adjustment number, sets status to draft, and adds items.
/// Requirements: 9.4, 9.8
pub struct CreateAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    adjustment_repo: Arc<A>,
}

impl<A> CreateAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    /// Creates a new instance of CreateAdjustmentUseCase
    pub fn new(adjustment_repo: Arc<A>) -> Self {
        Self { adjustment_repo }
    }

    /// Executes the use case to create a new stock adjustment
    ///
    /// # Arguments
    /// * `command` - The create adjustment command containing adjustment data
    /// * `actor_id` - ID of the user creating the adjustment
    ///
    /// # Returns
    /// AdjustmentDetailResponse on success
    ///
    /// # Errors
    /// * `InventoryError::InvalidAdjustmentType` - If adjustment type is invalid
    /// * `InventoryError::InvalidAdjustmentReason` - If adjustment reason is invalid
    /// * `InventoryError::EmptyAdjustment` - If no items provided
    pub async fn execute(
        &self,
        command: CreateAdjustmentCommand,
        actor_id: UserId,
    ) -> Result<AdjustmentDetailResponse, InventoryError> {
        // 1. Parse and validate adjustment type and reason
        let adjustment_type = AdjustmentType::from_str(&command.adjustment_type)?;
        let adjustment_reason = AdjustmentReason::from_str(&command.adjustment_reason)?;

        // 2. Validate items are provided
        if command.items.is_empty() {
            return Err(InventoryError::EmptyAdjustment);
        }

        // 3. Generate adjustment number (Requirement 9.8)
        let store_id = StoreId::from_uuid(command.store_id);
        let adjustment_number = self
            .adjustment_repo
            .generate_adjustment_number(store_id)
            .await?;

        // 4. Create adjustment entity in draft status (Requirement 9.4)
        let mut adjustment = StockAdjustment::create(
            store_id,
            adjustment_number,
            adjustment_type,
            adjustment_reason,
            actor_id,
        );

        // 5. Set optional fields
        if let Some(notes) = command.notes {
            adjustment.set_notes(Some(notes))?;
        }
        if let Some(attachments) = command.attachments {
            adjustment.set_attachments(attachments)?;
        }

        // 6. Add items to adjustment
        for item_cmd in &command.items {
            let item = self.create_adjustment_item(adjustment.id(), item_cmd)?;
            adjustment.add_item(item)?;
        }

        // 7. Save adjustment
        self.adjustment_repo.save(&adjustment).await?;

        // 8. Convert to response
        Ok(self.to_response(&adjustment))
    }

    fn create_adjustment_item(
        &self,
        adjustment_id: crate::domain::value_objects::AdjustmentId,
        cmd: &AdjustmentItemCommand,
    ) -> Result<AdjustmentItem, InventoryError> {
        let stock_id = StockId::from_uuid(cmd.stock_id);
        let mut item = AdjustmentItem::create(adjustment_id, stock_id, cmd.quantity, cmd.unit_cost);

        if let Some(ref notes) = cmd.notes {
            item.set_notes(Some(notes.clone()));
        }

        Ok(item)
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

    use crate::domain::value_objects::AdjustmentId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    struct MockAdjustmentRepository {
        adjustments: Mutex<HashMap<AdjustmentId, StockAdjustment>>,
        counter: Mutex<i32>,
    }

    impl MockAdjustmentRepository {
        fn new() -> Self {
            Self {
                adjustments: Mutex::new(HashMap::new()),
                counter: Mutex::new(0),
            }
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
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            Ok(format!("ADJ-TEST-{:05}", *counter))
        }
    }

    #[tokio::test]
    async fn test_create_adjustment_success() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = CreateAdjustmentUseCase::new(repo.clone());

        let command = CreateAdjustmentCommand {
            store_id: new_uuid(),
            adjustment_type: "decrease".to_string(),
            adjustment_reason: "damage".to_string(),
            notes: Some("Damaged goods".to_string()),
            attachments: None,
            items: vec![AdjustmentItemCommand {
                stock_id: new_uuid(),
                quantity: dec!(-10),
                unit_cost: Some(dec!(5.00)),
                notes: Some("Item notes".to_string()),
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.adjustment_number, "ADJ-TEST-00001");
        assert_eq!(response.adjustment_type, "decrease");
        assert_eq!(response.adjustment_reason, "damage");
        assert_eq!(response.status, "draft");
        assert_eq!(response.notes, Some("Damaged goods".to_string()));
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].quantity, dec!(-10));
    }

    #[tokio::test]
    async fn test_create_adjustment_multiple_items() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = CreateAdjustmentUseCase::new(repo);

        let command = CreateAdjustmentCommand {
            store_id: new_uuid(),
            adjustment_type: "increase".to_string(),
            adjustment_reason: "found".to_string(),
            notes: None,
            attachments: None,
            items: vec![
                AdjustmentItemCommand {
                    stock_id: new_uuid(),
                    quantity: dec!(5),
                    unit_cost: None,
                    notes: None,
                },
                AdjustmentItemCommand {
                    stock_id: new_uuid(),
                    quantity: dec!(10),
                    unit_cost: Some(dec!(15.00)),
                    notes: None,
                },
            ],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items.len(), 2);
    }

    #[tokio::test]
    async fn test_create_adjustment_empty_items() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = CreateAdjustmentUseCase::new(repo);

        let command = CreateAdjustmentCommand {
            store_id: new_uuid(),
            adjustment_type: "decrease".to_string(),
            adjustment_reason: "damage".to_string(),
            notes: None,
            attachments: None,
            items: vec![],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::EmptyAdjustment)));
    }

    #[tokio::test]
    async fn test_create_adjustment_invalid_type() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = CreateAdjustmentUseCase::new(repo);

        let command = CreateAdjustmentCommand {
            store_id: new_uuid(),
            adjustment_type: "invalid".to_string(),
            adjustment_reason: "damage".to_string(),
            notes: None,
            attachments: None,
            items: vec![AdjustmentItemCommand {
                stock_id: new_uuid(),
                quantity: dec!(-10),
                unit_cost: None,
                notes: None,
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::InvalidAdjustmentType)));
    }

    #[tokio::test]
    async fn test_create_adjustment_invalid_reason() {
        let repo = Arc::new(MockAdjustmentRepository::new());
        let use_case = CreateAdjustmentUseCase::new(repo);

        let command = CreateAdjustmentCommand {
            store_id: new_uuid(),
            adjustment_type: "decrease".to_string(),
            adjustment_reason: "invalid".to_string(),
            notes: None,
            attachments: None,
            items: vec![AdjustmentItemCommand {
                stock_id: new_uuid(),
                quantity: dec!(-10),
                unit_cost: None,
                notes: None,
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidAdjustmentReason)
        ));
    }
}
