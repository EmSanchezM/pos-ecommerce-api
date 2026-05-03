//! Service order workflow transitions (excluding quote-driven ones).

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use service_orders::{
    CancelServiceOrderCommand, CancelServiceOrderUseCase, DeliverServiceOrderUseCase,
    DiagnoseServiceOrderUseCase, MarkReadyUseCase, ServiceOrderId, ServiceOrderResponse,
    StartRepairUseCase, StartTestingUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_feature;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

macro_rules! transition_handler {
    ($fn_name:ident, $use_case:ident, $perm:expr) => {
        pub async fn $fn_name(
            State(state): State<AppState>,
            CurrentUser(ctx): CurrentUser,
            Path(id): Path<Uuid>,
        ) -> Result<Json<ServiceOrderResponse>, Response> {
            require_permission(&ctx, $perm)?;
            require_feature(state.pool(), &ctx, "service_orders").await?;
            let use_case = $use_case::new(state.service_order_repo());
            let order = use_case
                .execute(ServiceOrderId::from_uuid(id))
                .await
                .map_err(|e| AppError::from(e).into_response())?;
            Ok(Json(ServiceOrderResponse::from(&order)))
        }
    };
}

transition_handler!(
    diagnose_service_order_handler,
    DiagnoseServiceOrderUseCase,
    "service_orders:transition_order"
);
transition_handler!(
    start_repair_handler,
    StartRepairUseCase,
    "service_orders:transition_order"
);
transition_handler!(
    start_testing_handler,
    StartTestingUseCase,
    "service_orders:transition_order"
);
transition_handler!(
    mark_ready_handler,
    MarkReadyUseCase,
    "service_orders:transition_order"
);
transition_handler!(
    deliver_service_order_handler,
    DeliverServiceOrderUseCase,
    "service_orders:transition_order"
);

pub async fn cancel_service_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<CancelServiceOrderCommand>,
) -> Result<Json<ServiceOrderResponse>, Response> {
    require_permission(&ctx, "service_orders:cancel_order")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case = CancelServiceOrderUseCase::new(state.service_order_repo());
    let order = use_case
        .execute(ServiceOrderId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceOrderResponse::from(&order)))
}
