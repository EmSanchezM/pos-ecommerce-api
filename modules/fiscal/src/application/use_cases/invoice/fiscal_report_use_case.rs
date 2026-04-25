//! Fiscal report use case

use crate::FiscalError;
use crate::application::dtos::FiscalReportCommand;
use crate::application::dtos::FiscalReportResponse;

/// Use case for generating a fiscal report for a date range (placeholder)
#[derive(Default)]
pub struct FiscalReportUseCase;

impl FiscalReportUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _cmd: FiscalReportCommand,
    ) -> Result<FiscalReportResponse, FiscalError> {
        Err(FiscalError::NotImplemented)
    }
}
