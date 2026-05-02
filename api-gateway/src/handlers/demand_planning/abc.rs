//! ABC classification read endpoint.

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use demand_planning::{AbcClass, AbcClassificationResponse, ListAbcClassificationsUseCase};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListAbcQuery {
    pub store_id: Option<Uuid>,
    pub class: Option<AbcClass>,
}

pub async fn list_abc_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListAbcQuery>,
) -> Result<Json<Vec<AbcClassificationResponse>>, Response> {
    require_permission(&ctx, "demand_planning:read_abc")?;
    let use_case = ListAbcClassificationsUseCase::new(state.abc_classification_repo());
    let classifications = use_case
        .execute(params.store_id, params.class)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        classifications
            .iter()
            .map(AbcClassificationResponse::from)
            .collect(),
    ))
}
