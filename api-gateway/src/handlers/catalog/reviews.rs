// Review handlers — submit (customer), approve/delete (moderator), pending list.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use catalog::{
    ApproveReviewUseCase, DeleteReviewUseCase, ListPendingReviewsUseCase, ReviewListResponse,
    ReviewResponse, SubmitReviewCommand, SubmitReviewUseCase,
};

pub async fn submit_review_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(listing_id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<SubmitReviewCommand>,
) -> Result<(StatusCode, Json<ReviewResponse>), Response> {
    require_permission(&ctx, "catalog:review")?;
    cmd.listing_id = listing_id;
    let uc = SubmitReviewUseCase::new(state.review_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn approve_review_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(review_id): Path<Uuid>,
) -> Result<Json<ReviewResponse>, Response> {
    require_permission(&ctx, "catalog:moderate")?;
    let uc = ApproveReviewUseCase::new(state.review_repo());
    let resp = uc
        .execute(review_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_review_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(review_id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "catalog:moderate")?;
    let uc = DeleteReviewUseCase::new(state.review_repo());
    uc.execute(review_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct PendingReviewsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_pending_reviews_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<PendingReviewsQuery>,
) -> Result<Json<ReviewListResponse>, Response> {
    require_permission(&ctx, "catalog:moderate")?;
    let uc = ListPendingReviewsUseCase::new(state.review_repo());
    let resp = uc
        .execute(q.page, q.page_size)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
