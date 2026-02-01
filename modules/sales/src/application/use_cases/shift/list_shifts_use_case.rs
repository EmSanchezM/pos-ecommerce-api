//! List shifts use case

use std::str::FromStr;
use std::sync::Arc;

use chrono::NaiveDate;

use crate::application::dtos::{ListShiftsQuery, ShiftListResponse, ShiftResponse};
use crate::domain::repositories::{ShiftFilter, ShiftRepository};
use crate::domain::value_objects::ShiftStatus;
use crate::SalesError;
use identity::{StoreId, UserId};
use pos_core::TerminalId;

/// Use case for listing shifts with filters and pagination
pub struct ListShiftsUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl ListShiftsUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    pub async fn execute(&self, query: ListShiftsQuery) -> Result<ShiftListResponse, SalesError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100).max(1);

        let filter = ShiftFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            terminal_id: query.terminal_id.map(TerminalId::from_uuid),
            cashier_id: query.cashier_id.map(UserId::from_uuid),
            status: query
                .status
                .as_ref()
                .and_then(|s| ShiftStatus::from_str(s).ok()),
            date_from: query
                .date_from
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
            date_to: query
                .date_to
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        };

        let (shifts, total) = self.shift_repo.find_paginated(filter, page, page_size).await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(ShiftListResponse {
            data: shifts.into_iter().map(ShiftResponse::from).collect(),
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
