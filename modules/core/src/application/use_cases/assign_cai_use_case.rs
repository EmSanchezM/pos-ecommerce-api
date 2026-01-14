// AssignCaiUseCase - Assigns a CAI range to a terminal
//
// Requirements: 2.2, 2.3, 6.3
// - Validate terminal exists and is active
// - Validate CAI format and range validity
// - Verify no overlap with existing active ranges
// - Save new CAI range
// - Register audit entry

use std::sync::Arc;

use chrono::Utc;
use identity::{AuditEntry, AuditRepository, UserId};
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::entities::CaiRange;
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::{CaiNumber, TerminalId};
use crate::error::CoreError;
use crate::AssignCaiCommand;

/// Use case for assigning a CAI range to a terminal
///
/// This use case orchestrates the assignment of a CAI range to a terminal,
/// ensuring all business rules are satisfied:
/// - Terminal must exist and be active
/// - CAI number format must be valid
/// - Range must be valid (start <= end)
/// - No overlap with existing active CAI ranges
/// - Audit trail is recorded
pub struct AssignCaiUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    terminal_repo: Arc<T>,
    audit_repo: Arc<A>,
}

impl<T, A> AssignCaiUseCase<T, A>
where
    T: TerminalRepository,
    A: AuditRepository,
{
    /// Creates a new instance of AssignCaiUseCase
    pub fn new(terminal_repo: Arc<T>, audit_repo: Arc<A>) -> Self {
        Self {
            terminal_repo,
            audit_repo,
        }
    }

    /// Executes the use case to assign a CAI range to a terminal
    ///
    /// # Arguments
    /// * `command` - The command containing CAI assignment data
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(CaiRange)` - The created CAI range
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    /// * `Err(CoreError::TerminalInactive)` - If the terminal is inactive
    /// * `Err(CoreError::InvalidCaiNumber)` - If the CAI number format is invalid
    /// * `Err(CoreError::InvalidCaiRange)` - If range_start > range_end
    /// * `Err(CoreError::CaiRangeOverlap)` - If the range overlaps with an active range
    pub async fn execute(
        &self,
        command: AssignCaiCommand,
        actor_id: UserId,
    ) -> Result<CaiRange, CoreError> {
        let terminal_id = TerminalId::from_uuid(command.terminal_id);

        // 1. Verify terminal exists and is active
        let mut terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(command.terminal_id))?;

        if !terminal.is_active() {
            return Err(CoreError::TerminalInactive(command.terminal_id));
        }

        // 2. Validate CAI number format
        let cai_number = CaiNumber::new(&command.cai_number)?;

        // 3. Validate range (start <= end)
        if command.range_start > command.range_end {
            return Err(CoreError::InvalidCaiRange);
        }

        // 4. Check for overlap with existing active CAI ranges
        let cai_history = self.terminal_repo.get_cai_history(terminal_id).await?;
        for existing_cai in &cai_history {
            // Only check non-exhausted, non-expired ranges
            if !existing_cai.is_exhausted_flag() && !existing_cai.is_expired() {
                // Check for range overlap
                if Self::ranges_overlap(
                    command.range_start,
                    command.range_end,
                    existing_cai.range_start(),
                    existing_cai.range_end(),
                ) {
                    return Err(CoreError::CaiRangeOverlap);
                }
            }
        }

        // 5. Create new CAI range
        let cai_range = CaiRange::new(
            Uuid::new_v7(Timestamp::now(NoContext)),
            cai_number,
            command.range_start,
            command.range_end,
            command.range_start, // current_number starts at range_start
            command.expiration_date,
            false, // not exhausted
            Utc::now(),
        );

        // 6. Save CAI range to repository
        self.terminal_repo
            .save_cai_range(terminal_id, &cai_range)
            .await?;

        // 7. Update terminal with new CAI
        terminal.set_cai(cai_range.clone());
        self.terminal_repo.update(&terminal).await?;

        // 8. Audit
        let audit = AuditEntry::for_create(
            "cai_range",
            cai_range.id(),
            &cai_range,
            actor_id,
        );
        self.audit_repo
            .save(&audit)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(cai_range)
    }

    /// Checks if two ranges overlap
    fn ranges_overlap(start1: i64, end1: i64, start2: i64, end2: i64) -> bool {
        start1 <= end2 && start2 <= end1
    }
}
