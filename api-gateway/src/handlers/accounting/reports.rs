//! GET /api/v1/accounting/reports/profit-and-loss?period_id=&store_id=

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use accounting::{AccountingPeriodId, GenerateProfitAndLossUseCase, ProfitAndLossResponse};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PnlQuery {
    pub period_id: Uuid,
    pub store_id: Option<Uuid>,
}

pub async fn profit_and_loss_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<PnlQuery>,
) -> Result<Json<ProfitAndLossResponse>, Response> {
    require_permission(&ctx, "accounting:read")?;

    let use_case = GenerateProfitAndLossUseCase::new(state.accounting_report_repo());
    let response = use_case
        .execute(
            AccountingPeriodId::from_uuid(params.period_id),
            params.store_id,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
