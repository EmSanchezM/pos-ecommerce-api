// UpdateTerminalUseCase - Updates an existing terminal
//
// - Find existing terminal
// - Update specified fields
// - Register audit with old and new values

use std::sync::Arc;

use identity::{AuditEntry, AuditRepository, UserId};

use crate::domain::entities::Terminal;
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalId;
use crate::error::CoreError;
use crate::UpdateTerminalCommand;

/// Use case for updating an existing terminal
///
/// This use case orchestrates the update of a terminal,
/// ensuring all business rules are satisfied:
/// - Terminal must exist
/// - Only specified fields are updated
/// - Audit trail records old and new values
pub struct UpdateTerminalUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    terminal_repo: Arc<T>,
    audit_repo: Arc<A>,
}

impl<T, A> UpdateTerminalUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateTerminalUseCase
    pub fn new(terminal_repo: Arc<T>, audit_repo: Arc<A>) -> Self {
        Self {
            terminal_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update a terminal
    ///
    /// # Arguments
    /// * `terminal_id` - The ID of the terminal to update
    /// * `command` - The command containing fields to update
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Terminal)` - The updated terminal
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    pub async fn execute(
        &self,
        terminal_id: TerminalId,
        command: UpdateTerminalCommand,
        actor_id: UserId,
    ) -> Result<Terminal, CoreError> {
        // 1. Find existing terminal
        let mut terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(terminal_id.into_uuid()))?;

        // 2. Clone old state for audit
        let old_terminal = terminal.clone();

        // 3. Update specified fields
        let mut updated = false;
        if let Some(name) = command.name {
            terminal.set_name(name);
            updated = true;
        }

        // 4. Save if any changes were made
        if updated {
            self.terminal_repo.update(&terminal).await?;

            // 5. Audit with old and new values
            let audit = AuditEntry::for_update(
                "terminal",
                terminal.id().into_uuid(),
                &old_terminal,
                &terminal,
                actor_id,
            );
            self.audit_repo
                .save(&audit)
                .await
                .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;
        }

        Ok(terminal)
    }
}
