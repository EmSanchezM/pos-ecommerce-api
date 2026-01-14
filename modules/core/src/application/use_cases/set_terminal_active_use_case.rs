// SetTerminalActiveUseCase - Activates or deactivates a terminal
//
// Requirements: 2.6, 6.2
// - Activate or deactivate terminal
// - Preserve CAI history when deactivating
// - Register audit entry

use std::sync::Arc;

use identity::{AuditEntry, AuditRepository, UserId};

use crate::domain::entities::Terminal;
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalId;
use crate::error::CoreError;

/// Use case for activating or deactivating a terminal
///
/// This use case orchestrates the activation/deactivation of a terminal,
/// ensuring all business rules are satisfied:
/// - Terminal must exist
/// - CAI history is preserved when deactivating
/// - Audit trail is recorded
pub struct SetTerminalActiveUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    terminal_repo: Arc<T>,
    audit_repo: Arc<A>,
}

impl<T, A> SetTerminalActiveUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    /// Creates a new instance of SetTerminalActiveUseCase
    pub fn new(terminal_repo: Arc<T>, audit_repo: Arc<A>) -> Self {
        Self {
            terminal_repo,
            audit_repo,
        }
    }

    /// Executes the use case to activate a terminal
    ///
    /// # Arguments
    /// * `terminal_id` - The ID of the terminal to activate
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Terminal)` - The activated terminal
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    pub async fn activate(
        &self,
        terminal_id: TerminalId,
        actor_id: UserId,
    ) -> Result<Terminal, CoreError> {
        self.set_active(terminal_id, true, actor_id).await
    }

    /// Executes the use case to deactivate a terminal
    ///
    /// CAI history is preserved when deactivating.
    ///
    /// # Arguments
    /// * `terminal_id` - The ID of the terminal to deactivate
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Terminal)` - The deactivated terminal
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    pub async fn deactivate(
        &self,
        terminal_id: TerminalId,
        actor_id: UserId,
    ) -> Result<Terminal, CoreError> {
        self.set_active(terminal_id, false, actor_id).await
    }

    /// Internal method to set terminal active status
    async fn set_active(
        &self,
        terminal_id: TerminalId,
        active: bool,
        actor_id: UserId,
    ) -> Result<Terminal, CoreError> {
        // 1. Find existing terminal
        let mut terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(terminal_id.into_uuid()))?;

        // 2. Check if state change is needed
        if terminal.is_active() == active {
            return Ok(terminal);
        }

        // 3. Clone old state for audit
        let old_terminal = terminal.clone();

        // 4. Update active status
        // Note: CAI history is preserved in the database (cai_ranges table)
        // The terminal's current_cai remains unchanged
        if active {
            terminal.activate();
        } else {
            terminal.deactivate();
        }

        // 5. Save changes
        self.terminal_repo.update(&terminal).await?;

        // 6. Audit
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

        Ok(terminal)
    }
}
