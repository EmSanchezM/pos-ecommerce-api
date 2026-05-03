use async_trait::async_trait;
use chrono::NaiveTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::ResourceCalendar;
use crate::domain::repositories::ResourceCalendarRepository;
use crate::domain::value_objects::{ResourceCalendarId, ResourceId};

pub struct PgResourceCalendarRepository {
    pool: PgPool,
}

impl PgResourceCalendarRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResourceCalendarRepository for PgResourceCalendarRepository {
    async fn replace_for_resource(
        &self,
        resource_id: ResourceId,
        entries: &[ResourceCalendar],
    ) -> Result<(), BookingError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM booking_resource_calendars WHERE resource_id = $1")
            .bind(resource_id.into_uuid())
            .execute(&mut *tx)
            .await?;
        for c in entries {
            sqlx::query(
                r#"
                INSERT INTO booking_resource_calendars (
                    id, resource_id, day_of_week, start_time, end_time, is_active
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(c.id().into_uuid())
            .bind(c.resource_id().into_uuid())
            .bind(c.day_of_week())
            .bind(c.start_time())
            .bind(c.end_time())
            .bind(c.is_active())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn find_by_resource(
        &self,
        resource_id: ResourceId,
    ) -> Result<Vec<ResourceCalendar>, BookingError> {
        let rows = sqlx::query_as::<_, CalendarRow>(
            r#"
            SELECT id, resource_id, day_of_week, start_time, end_time, is_active
            FROM booking_resource_calendars
            WHERE resource_id = $1
            ORDER BY day_of_week, start_time
            "#,
        )
        .bind(resource_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(ResourceCalendar::from).collect())
    }
}

#[derive(sqlx::FromRow)]
struct CalendarRow {
    id: Uuid,
    resource_id: Uuid,
    day_of_week: i16,
    start_time: NaiveTime,
    end_time: NaiveTime,
    is_active: bool,
}

impl From<CalendarRow> for ResourceCalendar {
    fn from(r: CalendarRow) -> Self {
        ResourceCalendar::reconstitute(
            ResourceCalendarId::from_uuid(r.id),
            ResourceId::from_uuid(r.resource_id),
            r.day_of_week,
            r.start_time,
            r.end_time,
            r.is_active,
        )
    }
}
