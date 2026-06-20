use std::sync::Arc;

use chrono::Utc;
use sqlx::{Postgres, Transaction};

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::{CancelSubscriptionCommand, SubscriptionResponse};
use crate::domain::repositories::SubscriptionRepository;

pub struct CancelSubscriptionUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
}

impl CancelSubscriptionUseCase {
    pub fn new(sub_repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { sub_repo }
    }

    pub async fn execute(
        &self,
        cmd: CancelSubscriptionCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let mut subscription = self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(cmd.organization_id))?;

        let now = Utc::now();
        // `at_period_end = !immediately`: defer to the billing job by default;
        // immediately is reserved for super_admin in the handler layer.
        subscription.cancel(!cmd.immediately, now)?;
        self.sub_repo.update_with_version(&subscription).await?;

        // TODO(events): publish `subscription.canceled`.
        Ok(SubscriptionResponse::from(subscription))
    }

    /// Transactional path — load/cancel/persist inside the caller's tx so the
    /// cancellation commits atomically with an audit-outbox event.
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cmd: CancelSubscriptionCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let mut subscription = self
            .sub_repo
            .find_active_by_organization_in_tx(tx, org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(cmd.organization_id))?;

        let now = Utc::now();
        subscription.cancel(!cmd.immediately, now)?;
        self.sub_repo
            .update_with_version_in_tx(tx, &subscription)
            .await?;

        Ok(SubscriptionResponse::from(subscription))
    }
}
