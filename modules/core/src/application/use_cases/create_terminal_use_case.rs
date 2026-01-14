// CreateTerminalUseCase - Creates a new terminal for a store
//
// - Verify store exists and is active
// - Validate terminal code is unique within the store
// - Create terminal and save
// - Register audit entry

use std::sync::Arc;

use identity::{AuditEntry, AuditRepository, StoreRepository, UserId};

use crate::domain::entities::Terminal;
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalCode;
use crate::error::CoreError;
use crate::CreateTerminalCommand;

/// Use case for creating a new terminal
///
/// This use case orchestrates the creation of a terminal for a store,
/// ensuring all business rules are satisfied:
/// - Store must exist and be active
/// - Terminal code must be unique within the store
/// - Audit trail is recorded
pub struct CreateTerminalUseCase<T, S, A>
where
    T: TerminalRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    terminal_repo: Arc<T>,
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<T, S, A> CreateTerminalUseCase<T, S, A>
where
    T: TerminalRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreateTerminalUseCase
    pub fn new(terminal_repo: Arc<T>, store_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            terminal_repo,
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new terminal
    ///
    /// # Arguments
    /// * `command` - The command containing terminal creation data
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Terminal)` - The created terminal
    /// * `Err(CoreError::StoreNotFound)` - If the store doesn't exist
    /// * `Err(CoreError::StoreInactive)` - If the store is inactive
    /// * `Err(CoreError::InvalidTerminalCode)` - If the code format is invalid
    /// * `Err(CoreError::TerminalCodeExists)` - If the code already exists in the store
    pub async fn execute(
        &self,
        command: CreateTerminalCommand,
        actor_id: UserId,
    ) -> Result<Terminal, CoreError> {
        // 1. Verify store exists and is active
        let store_id = identity::StoreId::from_uuid(command.store_id);
        let store = self
            .store_repo
            .find_by_id(store_id)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?
            .ok_or(CoreError::StoreNotFound(command.store_id))?;

        if !store.is_active() {
            return Err(CoreError::StoreInactive(command.store_id));
        }

        // 2. Validate and create terminal code
        let code = TerminalCode::new(&command.code)?;

        // 3. Check code uniqueness within store
        if self
            .terminal_repo
            .find_by_code(store_id, &code)
            .await?
            .is_some()
        {
            return Err(CoreError::TerminalCodeExists(command.code));
        }

        // 4. Create terminal
        let terminal = Terminal::create(store_id, code, command.name);

        // 5. Save terminal
        self.terminal_repo.save(&terminal).await?;

        // 6. Audit
        let audit = AuditEntry::for_create("terminal", terminal.id().into_uuid(), &terminal, actor_id);
        self.audit_repo
            .save(&audit)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(terminal)
    }
}
