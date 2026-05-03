//! EnrollMemberUseCase — creates the program-customer link, validating that
//! both sides exist and that the customer isn't already enrolled. Customer
//! existence is checked via SQL (sales::Customer is a separate module; we
//! don't depend on its repos).

use std::sync::Arc;

use sqlx::PgPool;

use crate::LoyaltyError;
use crate::application::dtos::EnrollMemberCommand;
use crate::domain::entities::LoyaltyMember;
use crate::domain::repositories::{LoyaltyMemberRepository, LoyaltyProgramRepository};

pub struct EnrollMemberUseCase {
    programs: Arc<dyn LoyaltyProgramRepository>,
    members: Arc<dyn LoyaltyMemberRepository>,
    pool: PgPool,
}

impl EnrollMemberUseCase {
    pub fn new(
        programs: Arc<dyn LoyaltyProgramRepository>,
        members: Arc<dyn LoyaltyMemberRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            programs,
            members,
            pool,
        }
    }

    pub async fn execute(&self, cmd: EnrollMemberCommand) -> Result<LoyaltyMember, LoyaltyError> {
        self.programs
            .find_by_id(cmd.program_id)
            .await?
            .ok_or_else(|| LoyaltyError::ProgramNotFound(cmd.program_id.into_uuid()))?;

        // Verify customer exists in the sales module's table.
        let customer_exists: Option<(uuid::Uuid,)> =
            sqlx::query_as("SELECT id FROM customers WHERE id = $1")
                .bind(cmd.customer_id)
                .fetch_optional(&self.pool)
                .await?;
        if customer_exists.is_none() {
            return Err(LoyaltyError::CustomerNotFound(cmd.customer_id));
        }

        if self
            .members
            .find_by_customer(cmd.program_id, cmd.customer_id)
            .await?
            .is_some()
        {
            return Err(LoyaltyError::AlreadyEnrolled {
                program_id: cmd.program_id.into_uuid(),
            });
        }

        let member = LoyaltyMember::enroll(cmd.program_id, cmd.customer_id);
        self.members.save(&member).await?;
        Ok(member)
    }
}
