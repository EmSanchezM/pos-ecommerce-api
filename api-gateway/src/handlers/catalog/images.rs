// Image handlers.
//
// Upload uses multipart/form-data: a `file` part with the image bytes plus
// optional `alt_text` and `is_primary` fields.

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use catalog::{
    DeleteImageUseCase, ImageResponse, ListImagesUseCase, ReorderImagesCommand,
    ReorderImagesUseCase, UpdateImageCommand, UpdateImageUseCase, UploadImageCommand,
    UploadImageUseCase,
};
use identity::ErrorResponse;

pub async fn list_images_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(listing_id): Path<Uuid>,
) -> Result<Json<Vec<ImageResponse>>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = ListImagesUseCase::new(state.catalog_image_repo());
    let resp = uc
        .execute(listing_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

/// multipart/form-data:
///   field `file`         — required, bytes
///   field `alt_text`     — optional
///   field `is_primary`   — optional ("true"/"false")
pub async fn upload_image_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(listing_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ImageResponse>), Response> {
    require_permission(&ctx, "catalog:update")?;

    let mut bytes: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut alt_text: Option<String> = None;
    let mut is_primary = false;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_MULTIPART",
                format!("Invalid multipart payload: {}", e),
            )),
        )
            .into_response()
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().map(str::to_string);
                content_type = field.content_type().map(str::to_string);
                let data = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new(
                            "INVALID_MULTIPART_FILE",
                            format!("Failed to read file part: {}", e),
                        )),
                    )
                        .into_response()
                })?;
                bytes = Some(data.to_vec());
            }
            "alt_text" => {
                alt_text = field.text().await.ok();
            }
            "is_primary" => {
                is_primary = field
                    .text()
                    .await
                    .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
                    .unwrap_or(false);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let bytes = bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "MISSING_FILE",
                "Multipart field `file` is required",
            )),
        )
            .into_response()
    })?;

    let cmd = UploadImageCommand {
        listing_id,
        bytes,
        original_filename: filename.unwrap_or_else(|| "upload.bin".to_string()),
        content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        alt_text,
        is_primary,
    };

    let uc = UploadImageUseCase::new(
        state.listing_repo(),
        state.catalog_image_repo(),
        state.image_storage_provider_repo(),
        state.image_storage_registry(),
    );
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_image_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(image_id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateImageCommand>,
) -> Result<Json<ImageResponse>, Response> {
    require_permission(&ctx, "catalog:update")?;
    cmd.image_id = image_id;
    let uc = UpdateImageUseCase::new(state.catalog_image_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_image_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(image_id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "catalog:update")?;
    let uc = DeleteImageUseCase::new(
        state.catalog_image_repo(),
        state.image_storage_provider_repo(),
        state.image_storage_registry(),
    );
    uc.execute(image_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reorder_images_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(listing_id): Path<Uuid>,
    JsonBody(cmd): JsonBody<ReorderImagesCommand>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "catalog:update")?;
    let uc = ReorderImagesUseCase::new(state.catalog_image_repo());
    uc.execute(listing_id, cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
