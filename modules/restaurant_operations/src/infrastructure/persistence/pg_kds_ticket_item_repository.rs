use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::KdsTicketItem;
use crate::domain::repositories::KdsTicketItemRepository;
use crate::domain::value_objects::{KdsItemStatus, KdsTicketId, KdsTicketItemId};

pub struct PgKdsTicketItemRepository {
    pool: PgPool,
}

impl PgKdsTicketItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KdsTicketItemRepository for PgKdsTicketItemRepository {
    async fn save(&self, i: &KdsTicketItem) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO kds_ticket_items (
                id, ticket_id, sale_item_id, product_id,
                description, quantity, modifiers_summary, special_instructions,
                status, ready_at, served_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(i.id().into_uuid())
        .bind(i.ticket_id().into_uuid())
        .bind(i.sale_item_id())
        .bind(i.product_id())
        .bind(i.description())
        .bind(i.quantity())
        .bind(i.modifiers_summary())
        .bind(i.special_instructions())
        .bind(i.status().as_str())
        .bind(i.ready_at())
        .bind(i.served_at())
        .bind(i.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, i: &KdsTicketItem) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE kds_ticket_items
               SET status               = $2,
                   ready_at             = $3,
                   served_at            = $4,
                   modifiers_summary    = $5,
                   special_instructions = $6
             WHERE id = $1
            "#,
        )
        .bind(i.id().into_uuid())
        .bind(i.status().as_str())
        .bind(i.ready_at())
        .bind(i.served_at())
        .bind(i.modifiers_summary())
        .bind(i.special_instructions())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::ItemNotFound(i.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: KdsTicketItemId,
    ) -> Result<Option<KdsTicketItem>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, ItemRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(KdsTicketItem::try_from).transpose()
    }

    async fn list_by_ticket(
        &self,
        ticket_id: KdsTicketId,
    ) -> Result<Vec<KdsTicketItem>, RestaurantOperationsError> {
        let rows = sqlx::query_as::<_, ItemRow>(LIST_BY_TICKET)
            .bind(ticket_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(KdsTicketItem::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, ticket_id, sale_item_id, product_id,
       description, quantity, modifiers_summary, special_instructions,
       status, ready_at, served_at, created_at
FROM kds_ticket_items
WHERE id = $1
"#;

const LIST_BY_TICKET: &str = r#"
SELECT id, ticket_id, sale_item_id, product_id,
       description, quantity, modifiers_summary, special_instructions,
       status, ready_at, served_at, created_at
FROM kds_ticket_items
WHERE ticket_id = $1
ORDER BY created_at
"#;

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    ticket_id: Uuid,
    sale_item_id: Option<Uuid>,
    product_id: Option<Uuid>,
    description: String,
    quantity: Decimal,
    modifiers_summary: String,
    special_instructions: Option<String>,
    status: String,
    ready_at: Option<DateTime<Utc>>,
    served_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<ItemRow> for KdsTicketItem {
    type Error = RestaurantOperationsError;
    fn try_from(r: ItemRow) -> Result<Self, RestaurantOperationsError> {
        Ok(KdsTicketItem::reconstitute(
            KdsTicketItemId::from_uuid(r.id),
            KdsTicketId::from_uuid(r.ticket_id),
            r.sale_item_id,
            r.product_id,
            r.description,
            r.quantity,
            r.modifiers_summary,
            r.special_instructions,
            KdsItemStatus::from_str(&r.status)?,
            r.ready_at,
            r.served_at,
            r.created_at,
        ))
    }
}
