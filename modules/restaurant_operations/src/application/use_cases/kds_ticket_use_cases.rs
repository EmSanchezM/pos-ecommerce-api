//! KDS ticket lifecycle. Every mutation that the kitchen cares about
//! publishes a `KdsEvent` to the broadcaster so the SSE handler in the API
//! gateway can fan it out to every connected screen for that station.

use std::sync::Arc;

use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::application::broadcaster::{KdsBroadcaster, KdsEvent};
use crate::application::dtos::{
    CancelKdsTicketCommand, CreateKdsTicketCommand, SetItemStatusCommand,
};
use crate::domain::entities::{KdsTicket, KdsTicketItem};
use crate::domain::repositories::{
    KdsTicketItemRepository, KdsTicketRepository, KitchenStationRepository, ListKdsTicketsFilters,
    MenuModifierRepository, RestaurantTableRepository,
};
use crate::domain::value_objects::{
    KdsItemStatus, KdsTicketId, KdsTicketItemId, KdsTicketStatus, KitchenStationId, MenuModifierId,
    RestaurantTableId,
};

/// Bundles the dependencies shared by all KDS use cases. Construction in the
/// gateway stays a one-liner.
#[derive(Clone)]
pub struct KdsDeps {
    pub stations: Arc<dyn KitchenStationRepository>,
    pub tables: Arc<dyn RestaurantTableRepository>,
    pub modifiers: Arc<dyn MenuModifierRepository>,
    pub tickets: Arc<dyn KdsTicketRepository>,
    pub items: Arc<dyn KdsTicketItemRepository>,
    pub broadcaster: Arc<dyn KdsBroadcaster>,
}

pub struct CreateKdsTicketUseCase {
    deps: KdsDeps,
}

impl CreateKdsTicketUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        cmd: CreateKdsTicketCommand,
        actor_id: Option<Uuid>,
    ) -> Result<(KdsTicket, Vec<KdsTicketItem>), RestaurantOperationsError> {
        let station_id = KitchenStationId::from_uuid(cmd.station_id);
        let station = self
            .deps
            .stations
            .find_by_id(station_id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::StationNotFound(cmd.station_id))?;
        if station.store_id() != cmd.store_id {
            return Err(RestaurantOperationsError::Validation(
                "station does not belong to the supplied store".to_string(),
            ));
        }

        let table_id = match cmd.table_id {
            Some(t) => {
                let typed = RestaurantTableId::from_uuid(t);
                if self.deps.tables.find_by_id(typed).await?.is_none() {
                    return Err(RestaurantOperationsError::TableNotFound(t));
                }
                Some(typed)
            }
            None => None,
        };

        if cmd.items.is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "ticket needs at least one item".to_string(),
            ));
        }

        let ticket_number = self.deps.tickets.next_ticket_number(cmd.store_id).await?;
        let ticket = KdsTicket::new(
            cmd.store_id,
            station_id,
            table_id,
            cmd.sale_id,
            ticket_number,
            cmd.course.unwrap_or_default(),
            cmd.notes,
            actor_id,
        )?;
        self.deps.tickets.save(&ticket).await?;

        // Resolve modifiers in one round-trip per item to build the summary.
        let mut persisted_items = Vec::with_capacity(cmd.items.len());
        for item_dto in cmd.items {
            let modifier_summary = if item_dto.modifier_ids.is_empty() {
                String::new()
            } else {
                let typed: Vec<MenuModifierId> = item_dto
                    .modifier_ids
                    .iter()
                    .copied()
                    .map(MenuModifierId::from_uuid)
                    .collect();
                let modifiers = self.deps.modifiers.find_modifiers_in(&typed).await?;
                modifiers
                    .iter()
                    .map(|m| m.name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let item = KdsTicketItem::new(
                ticket.id(),
                item_dto.sale_item_id,
                item_dto.product_id,
                item_dto.description,
                item_dto.quantity,
                modifier_summary,
                item_dto.special_instructions,
            )?;
            self.deps.items.save(&item).await?;
            persisted_items.push(item);
        }

        // Pull the table label for the event payload (best-effort; if the
        // fetch fails we still emit the event with `None`).
        let table_label = match table_id {
            Some(tid) => self
                .deps
                .tables
                .find_by_id(tid)
                .await
                .ok()
                .flatten()
                .map(|t| t.label().to_string()),
            None => None,
        };
        self.deps
            .broadcaster
            .publish(
                station_id,
                KdsEvent::TicketCreated {
                    ticket_id: ticket.id().into_uuid(),
                    ticket_number: ticket.ticket_number(),
                    station_id: station_id.into_uuid(),
                    table_label,
                    items_count: persisted_items.len(),
                    course: ticket.course(),
                },
            )
            .await;

        Ok((ticket, persisted_items))
    }
}

pub struct ListKdsTicketsUseCase {
    tickets: Arc<dyn KdsTicketRepository>,
}

impl ListKdsTicketsUseCase {
    pub fn new(tickets: Arc<dyn KdsTicketRepository>) -> Self {
        Self { tickets }
    }

    pub async fn execute(
        &self,
        filters: ListKdsTicketsFilters,
    ) -> Result<Vec<KdsTicket>, RestaurantOperationsError> {
        self.tickets.list(filters).await
    }
}

pub struct GetKdsTicketUseCase {
    tickets: Arc<dyn KdsTicketRepository>,
    items: Arc<dyn KdsTicketItemRepository>,
}

impl GetKdsTicketUseCase {
    pub fn new(
        tickets: Arc<dyn KdsTicketRepository>,
        items: Arc<dyn KdsTicketItemRepository>,
    ) -> Self {
        Self { tickets, items }
    }

    pub async fn execute(
        &self,
        id: KdsTicketId,
    ) -> Result<(KdsTicket, Vec<KdsTicketItem>), RestaurantOperationsError> {
        let ticket = self
            .tickets
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(id.into_uuid()))?;
        let items = self.items.list_by_ticket(id).await?;
        Ok((ticket, items))
    }
}

pub struct SendKdsTicketUseCase {
    deps: KdsDeps,
}

impl SendKdsTicketUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: KdsTicketId) -> Result<KdsTicket, RestaurantOperationsError> {
        let mut ticket = self
            .deps
            .tickets
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(id.into_uuid()))?;
        ticket.send()?;
        self.deps.tickets.update(&ticket).await?;
        self.deps
            .broadcaster
            .publish(
                ticket.station_id(),
                KdsEvent::TicketStatusChanged {
                    ticket_id: ticket.id().into_uuid(),
                    ticket_number: ticket.ticket_number(),
                    status: ticket.status(),
                },
            )
            .await;
        Ok(ticket)
    }
}

pub struct MarkKdsTicketReadyUseCase {
    deps: KdsDeps,
}

impl MarkKdsTicketReadyUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: KdsTicketId) -> Result<KdsTicket, RestaurantOperationsError> {
        let mut ticket = self
            .deps
            .tickets
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(id.into_uuid()))?;
        ticket.mark_ready()?;
        self.deps.tickets.update(&ticket).await?;
        // Force every non-terminal item to Ready so the kitchen sees a
        // consistent state (cancellations/served items are left untouched).
        let items = self.deps.items.list_by_ticket(id).await?;
        for mut item in items {
            if matches!(
                item.status(),
                KdsItemStatus::Pending | KdsItemStatus::InProgress
            ) {
                if item.status() == KdsItemStatus::Pending {
                    item.transition_to(KdsItemStatus::InProgress)?;
                }
                item.transition_to(KdsItemStatus::Ready)?;
                self.deps.items.update(&item).await?;
            }
        }
        self.deps
            .broadcaster
            .publish(
                ticket.station_id(),
                KdsEvent::TicketStatusChanged {
                    ticket_id: ticket.id().into_uuid(),
                    ticket_number: ticket.ticket_number(),
                    status: ticket.status(),
                },
            )
            .await;
        Ok(ticket)
    }
}

pub struct ServeKdsTicketUseCase {
    deps: KdsDeps,
}

impl ServeKdsTicketUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: KdsTicketId) -> Result<KdsTicket, RestaurantOperationsError> {
        let mut ticket = self
            .deps
            .tickets
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(id.into_uuid()))?;
        ticket.serve()?;
        self.deps.tickets.update(&ticket).await?;
        // Mark every Ready item as Served too.
        let items = self.deps.items.list_by_ticket(id).await?;
        for mut item in items {
            if item.status() == KdsItemStatus::Ready {
                item.transition_to(KdsItemStatus::Served)?;
                self.deps.items.update(&item).await?;
            }
        }
        self.deps
            .broadcaster
            .publish(
                ticket.station_id(),
                KdsEvent::TicketStatusChanged {
                    ticket_id: ticket.id().into_uuid(),
                    ticket_number: ticket.ticket_number(),
                    status: ticket.status(),
                },
            )
            .await;
        Ok(ticket)
    }
}

pub struct CancelKdsTicketUseCase {
    deps: KdsDeps,
}

impl CancelKdsTicketUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: KdsTicketId,
        cmd: CancelKdsTicketCommand,
    ) -> Result<KdsTicket, RestaurantOperationsError> {
        let mut ticket = self
            .deps
            .tickets
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(id.into_uuid()))?;
        let reason = cmd.reason.clone();
        ticket.cancel(cmd.reason)?;
        self.deps.tickets.update(&ticket).await?;
        self.deps
            .broadcaster
            .publish(
                ticket.station_id(),
                KdsEvent::TicketCanceled {
                    ticket_id: ticket.id().into_uuid(),
                    ticket_number: ticket.ticket_number(),
                    reason,
                },
            )
            .await;
        Ok(ticket)
    }
}

/// Auto-advance the parent ticket when an item changes status.
///
/// Rules (only applied when the change leaves all items in the same status):
/// - all items InProgress and ticket Pending → ticket InProgress
/// - all items Ready and ticket InProgress → ticket Ready
/// - all items Served and ticket Ready → ticket Served (terminal)
///
/// Cancelled items are skipped (treated as already-resolved).
pub struct SetItemStatusUseCase {
    deps: KdsDeps,
}

impl SetItemStatusUseCase {
    pub fn new(deps: KdsDeps) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        item_id: KdsTicketItemId,
        cmd: SetItemStatusCommand,
    ) -> Result<KdsTicketItem, RestaurantOperationsError> {
        let mut item = self
            .deps
            .items
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::ItemNotFound(item_id.into_uuid()))?;
        let ticket_id = item.ticket_id();
        let mut ticket = self
            .deps
            .tickets
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TicketNotFound(ticket_id.into_uuid()))?;
        if ticket.status().is_terminal() {
            return Err(RestaurantOperationsError::CannotModifyTerminalTicket);
        }

        item.transition_to(cmd.status)?;
        self.deps.items.update(&item).await?;
        self.deps
            .broadcaster
            .publish(
                ticket.station_id(),
                KdsEvent::ItemStatusChanged {
                    ticket_id: ticket.id().into_uuid(),
                    item_id: item.id().into_uuid(),
                    status: item.status(),
                },
            )
            .await;

        // Re-fetch all items and check for unanimous advancement.
        let all = self.deps.items.list_by_ticket(ticket_id).await?;
        let interesting: Vec<&KdsTicketItem> = all
            .iter()
            .filter(|i| i.status() != KdsItemStatus::Canceled)
            .collect();
        if interesting.is_empty() {
            return Ok(item);
        }
        let auto_target = match cmd.status {
            KdsItemStatus::InProgress
                if ticket.status() == KdsTicketStatus::Pending
                    && interesting
                        .iter()
                        .all(|i| i.status() == KdsItemStatus::InProgress) =>
            {
                Some(KdsTicketStatus::InProgress)
            }
            KdsItemStatus::Ready
                if ticket.status() == KdsTicketStatus::InProgress
                    && interesting.iter().all(|i| {
                        matches!(i.status(), KdsItemStatus::Ready | KdsItemStatus::Served)
                    }) =>
            {
                Some(KdsTicketStatus::Ready)
            }
            KdsItemStatus::Served
                if ticket.status() == KdsTicketStatus::Ready
                    && interesting
                        .iter()
                        .all(|i| i.status() == KdsItemStatus::Served) =>
            {
                Some(KdsTicketStatus::Served)
            }
            _ => None,
        };
        if let Some(next) = auto_target {
            match next {
                KdsTicketStatus::InProgress => ticket.send()?,
                KdsTicketStatus::Ready => ticket.mark_ready()?,
                KdsTicketStatus::Served => ticket.serve()?,
                _ => unreachable!("auto_target only resolves to non-terminal forwards"),
            }
            self.deps.tickets.update(&ticket).await?;
            self.deps
                .broadcaster
                .publish(
                    ticket.station_id(),
                    KdsEvent::TicketStatusChanged {
                        ticket_id: ticket.id().into_uuid(),
                        ticket_number: ticket.ticket_number(),
                        status: ticket.status(),
                    },
                )
                .await;
        }
        Ok(item)
    }
}
