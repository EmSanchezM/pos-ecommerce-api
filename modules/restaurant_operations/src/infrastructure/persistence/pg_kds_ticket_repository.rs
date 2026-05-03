//! PgKdsTicketRepository — `next_ticket_number` runs the MAX query under
//! `SELECT ... FOR UPDATE` inside a transaction so two concurrent creates
//! cannot pick the same number for a store.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::KdsTicket;
use crate::domain::repositories::{KdsTicketRepository, ListKdsTicketsFilters};
use crate::domain::value_objects::{
    Course, KdsTicketId, KdsTicketStatus, KitchenStationId, RestaurantTableId,
};

pub struct PgKdsTicketRepository {
    pool: PgPool,
}

impl PgKdsTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KdsTicketRepository for PgKdsTicketRepository {
    async fn save(&self, t: &KdsTicket) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO kds_tickets (
                id, store_id, station_id, table_id, sale_id, ticket_number,
                status, course, notes, sent_at, ready_at, served_at,
                canceled_reason, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.store_id())
        .bind(t.station_id().into_uuid())
        .bind(t.table_id().map(|i| i.into_uuid()))
        .bind(t.sale_id())
        .bind(t.ticket_number())
        .bind(t.status().as_str())
        .bind(t.course().as_str())
        .bind(t.notes())
        .bind(t.sent_at())
        .bind(t.ready_at())
        .bind(t.served_at())
        .bind(t.canceled_reason())
        .bind(t.created_by())
        .bind(t.created_at())
        .bind(t.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, t: &KdsTicket) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE kds_tickets
               SET status          = $2,
                   notes           = $3,
                   sent_at          = $4,
                   ready_at         = $5,
                   served_at        = $6,
                   canceled_reason  = $7,
                   updated_at       = $8
             WHERE id = $1
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.status().as_str())
        .bind(t.notes())
        .bind(t.sent_at())
        .bind(t.ready_at())
        .bind(t.served_at())
        .bind(t.canceled_reason())
        .bind(t.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::TicketNotFound(
                t.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: KdsTicketId,
    ) -> Result<Option<KdsTicket>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, TicketRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(KdsTicket::try_from).transpose()
    }

    async fn list(
        &self,
        filters: ListKdsTicketsFilters,
    ) -> Result<Vec<KdsTicket>, RestaurantOperationsError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, store_id, station_id, table_id, sale_id, ticket_number, \
             status, course, notes, sent_at, ready_at, served_at, \
             canceled_reason, created_by, created_at, updated_at \
             FROM kds_tickets WHERE 1=1",
        );
        if let Some(store_id) = filters.store_id {
            qb.push(" AND store_id = ").push_bind(store_id);
        }
        if let Some(station_id) = filters.station_id {
            qb.push(" AND station_id = ")
                .push_bind(station_id.into_uuid());
        }
        if let Some(table_id) = filters.table_id {
            qb.push(" AND table_id = ").push_bind(table_id.into_uuid());
        }
        if let Some(status) = filters.status {
            qb.push(" AND status = ")
                .push_bind(status.as_str().to_string());
        }
        if let Some(from) = filters.from {
            qb.push(" AND created_at >= ").push_bind(from);
        }
        if let Some(to) = filters.to {
            qb.push(" AND created_at < ").push_bind(to);
        }
        qb.push(" ORDER BY created_at DESC");
        if let Some(limit) = filters.limit {
            qb.push(" LIMIT ").push_bind(limit);
        } else {
            qb.push(" LIMIT 200");
        }
        let rows: Vec<TicketRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(KdsTicket::try_from).collect()
    }

    async fn next_ticket_number(&self, store_id: Uuid) -> Result<i32, RestaurantOperationsError> {
        let mut tx = self.pool.begin().await?;
        // Lock existing rows for this store while we compute MAX+1. Postgres
        // 15 supports SELECT ... FOR UPDATE on the relation.
        let row: (Option<i32>,) = sqlx::query_as(
            "SELECT MAX(ticket_number) FROM kds_tickets WHERE store_id = $1 FOR UPDATE",
        )
        .bind(store_id)
        .fetch_one(&mut *tx)
        .await?;
        let next = row.0.unwrap_or(0) + 1;
        tx.commit().await?;
        Ok(next)
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, station_id, table_id, sale_id, ticket_number,
       status, course, notes, sent_at, ready_at, served_at,
       canceled_reason, created_by, created_at, updated_at
FROM kds_tickets
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct TicketRow {
    id: Uuid,
    store_id: Uuid,
    station_id: Uuid,
    table_id: Option<Uuid>,
    sale_id: Option<Uuid>,
    ticket_number: i32,
    status: String,
    course: String,
    notes: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    ready_at: Option<DateTime<Utc>>,
    served_at: Option<DateTime<Utc>>,
    canceled_reason: Option<String>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TicketRow> for KdsTicket {
    type Error = RestaurantOperationsError;
    fn try_from(r: TicketRow) -> Result<Self, RestaurantOperationsError> {
        Ok(KdsTicket::reconstitute(
            KdsTicketId::from_uuid(r.id),
            r.store_id,
            KitchenStationId::from_uuid(r.station_id),
            r.table_id.map(RestaurantTableId::from_uuid),
            r.sale_id,
            r.ticket_number,
            KdsTicketStatus::from_str(&r.status)?,
            Course::from_str(&r.course)?,
            r.notes,
            r.sent_at,
            r.ready_at,
            r.served_at,
            r.canceled_reason,
            r.created_by,
            r.created_at,
            r.updated_at,
        ))
    }
}
