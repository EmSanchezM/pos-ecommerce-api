//! SSE stream endpoint for the KDS. Each client subscribes per kitchen
//! station; every state change in any ticket of that station is streamed as a
//! single JSON-encoded event. Client reconnects re-subscribe to a fresh
//! receiver — broadcast channels do NOT replay history, so the kitchen
//! display should refetch the active ticket list right after connecting to
//! catch up on whatever happened while it was offline.

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::{
        Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::stream::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use restaurant_operations::KitchenStationId;

use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_feature;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn kds_stream_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(station_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    require_permission(&ctx, "restaurant:read_ticket")?;
    require_feature(state.pool(), &ctx, "restaurant").await?;

    let receiver = state
        .kds_broadcaster_handle()
        .subscribe(KitchenStationId::from_uuid(station_id))
        .await;

    let stream = BroadcastStream::new(receiver).filter_map(|res| async move {
        match res {
            Ok(event) => match Event::default().json_data(&event) {
                Ok(sse_event) => Some(Ok(sse_event)),
                Err(_) => None,
            },
            // `Lagged` means the channel buffer overflowed and we missed
            // events; we drop silently — the kitchen display refetches the
            // active list on reconnect to recover.
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}
