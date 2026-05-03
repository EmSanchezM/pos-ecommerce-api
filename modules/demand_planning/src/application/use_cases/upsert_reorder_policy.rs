//! UpsertReorderPolicyUseCase — creates a new policy or updates the existing
//! one for the (variant, store) tuple. Updates use optimistic locking on
//! `version`.

use std::sync::Arc;

use crate::DemandPlanningError;
use crate::application::dtos::UpsertReorderPolicyCommand;
use crate::domain::entities::ReorderPolicy;
use crate::domain::repositories::ReorderPolicyRepository;

pub struct UpsertReorderPolicyUseCase {
    repo: Arc<dyn ReorderPolicyRepository>,
}

impl UpsertReorderPolicyUseCase {
    pub fn new(repo: Arc<dyn ReorderPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: UpsertReorderPolicyCommand,
    ) -> Result<ReorderPolicy, DemandPlanningError> {
        let existing = self
            .repo
            .find_by_variant_store(cmd.product_variant_id, cmd.store_id)
            .await?;

        match existing {
            Some(mut policy) => {
                policy.update(
                    cmd.min_qty,
                    cmd.max_qty,
                    cmd.lead_time_days,
                    cmd.safety_stock_qty,
                    cmd.review_cycle_days,
                    cmd.preferred_vendor_id,
                )?;
                if !policy.is_active() {
                    policy.activate();
                }
                self.repo.update(&policy).await?;
                // The repo bumped `version + 1` in the UPDATE; mirror it in
                // memory so the caller (and the response DTO) sees the new
                // version they need to send on their next update.
                policy.increment_version();
                Ok(policy)
            }
            None => {
                let policy = ReorderPolicy::create(
                    cmd.product_variant_id,
                    cmd.store_id,
                    cmd.min_qty,
                    cmd.max_qty,
                    cmd.lead_time_days,
                    cmd.safety_stock_qty,
                    cmd.review_cycle_days,
                    cmd.preferred_vendor_id,
                )?;
                self.repo.save(&policy).await?;
                Ok(policy)
            }
        }
    }
}
