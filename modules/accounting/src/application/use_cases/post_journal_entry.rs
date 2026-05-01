//! PostJournalEntryUseCase — converts a `PostJournalEntryCommand` into a
//! posted JournalEntry.
//!
//! Validations:
//!   * Period must exist and be open.
//!   * At least two lines.
//!   * Each line has either debit or credit > 0 (enforced in `JournalLine::*_line`).
//!   * Sum of debits == sum of credits (enforced inside `JournalEntry::create`).
//!   * Every referenced account exists.
//!
//! `entry_number` is allocated by the repository as `max + 1` for the period.

use std::sync::Arc;

use crate::AccountingError;
use crate::application::dtos::{JournalLineCommand, LineSide, PostJournalEntryCommand};
use crate::domain::entities::{JournalEntry, JournalLine};
use crate::domain::repositories::{
    AccountRepository, AccountingPeriodRepository, JournalEntryRepository,
};

pub struct PostJournalEntryUseCase {
    accounts: Arc<dyn AccountRepository>,
    periods: Arc<dyn AccountingPeriodRepository>,
    entries: Arc<dyn JournalEntryRepository>,
}

impl PostJournalEntryUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        periods: Arc<dyn AccountingPeriodRepository>,
        entries: Arc<dyn JournalEntryRepository>,
    ) -> Self {
        Self {
            accounts,
            periods,
            entries,
        }
    }

    pub async fn execute(
        &self,
        cmd: PostJournalEntryCommand,
    ) -> Result<JournalEntry, AccountingError> {
        let period = self
            .periods
            .find_by_id(cmd.period_id)
            .await?
            .ok_or_else(|| AccountingError::PeriodNotFound(cmd.period_id.into_uuid()))?;
        if !period.is_open() {
            return Err(AccountingError::PeriodNotOpen);
        }

        // Validate every account referenced in the lines exists.
        for line in &cmd.lines {
            if self.accounts.find_by_id(line.account_id).await?.is_none() {
                return Err(AccountingError::AccountNotFound(
                    line.account_id.into_uuid(),
                ));
            }
        }

        let lines = build_lines(&cmd.lines)?;
        let entry_number = self.entries.next_entry_number(cmd.period_id).await?;
        let mut entry = JournalEntry::create(
            cmd.period_id,
            entry_number,
            cmd.description,
            cmd.source_type,
            cmd.source_id,
            cmd.created_by,
            lines,
        )?;

        entry.post()?;
        self.entries.save(&entry).await?;
        Ok(entry)
    }
}

fn build_lines(cmds: &[JournalLineCommand]) -> Result<Vec<JournalLine>, AccountingError> {
    let mut lines = Vec::with_capacity(cmds.len());
    for (idx, cmd) in cmds.iter().enumerate() {
        let line_number = (idx as i32) + 1;
        let line = match cmd.side {
            LineSide::Debit => JournalLine::debit_line(
                cmd.account_id,
                cmd.store_id,
                line_number,
                cmd.amount,
                cmd.description.clone(),
            )?,
            LineSide::Credit => JournalLine::credit_line(
                cmd.account_id,
                cmd.store_id,
                line_number,
                cmd.amount,
                cmd.description.clone(),
            )?,
        };
        lines.push(line);
    }
    Ok(lines)
}
