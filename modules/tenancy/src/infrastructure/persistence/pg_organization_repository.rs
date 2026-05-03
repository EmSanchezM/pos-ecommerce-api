use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::TenancyError;
use crate::domain::entities::Organization;
use crate::domain::repositories::OrganizationRepository;
use crate::domain::value_objects::{OrganizationId, OrganizationStatus};

pub struct PgOrganizationRepository {
    pool: PgPool,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    async fn save(&self, o: &Organization) -> Result<(), TenancyError> {
        sqlx::query(
            r#"
            INSERT INTO organizations (
                id, name, slug, contact_email, contact_phone,
                status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(o.id().into_uuid())
        .bind(o.name())
        .bind(o.slug())
        .bind(o.contact_email())
        .bind(o.contact_phone())
        .bind(o.status().as_str())
        .bind(o.created_at())
        .bind(o.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, o: &Organization) -> Result<(), TenancyError> {
        let result = sqlx::query(
            r#"
            UPDATE organizations
               SET name          = $2,
                   contact_email = $3,
                   contact_phone = $4,
                   status        = $5,
                   updated_at    = $6
             WHERE id = $1
            "#,
        )
        .bind(o.id().into_uuid())
        .bind(o.name())
        .bind(o.contact_email())
        .bind(o.contact_phone())
        .bind(o.status().as_str())
        .bind(o.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(TenancyError::OrganizationNotFound(o.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: OrganizationId) -> Result<Option<Organization>, TenancyError> {
        let row = sqlx::query_as::<_, OrgRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Organization::try_from).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, TenancyError> {
        let row = sqlx::query_as::<_, OrgRow>(SELECT_BY_SLUG)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Organization::try_from).transpose()
    }

    async fn list(&self, only_active: bool) -> Result<Vec<Organization>, TenancyError> {
        let sql = if only_active { LIST_ACTIVE } else { LIST_ALL };
        let rows = sqlx::query_as::<_, OrgRow>(sql)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Organization::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, name, slug, contact_email, contact_phone,
       status, created_at, updated_at
FROM organizations
WHERE id = $1
"#;

const SELECT_BY_SLUG: &str = r#"
SELECT id, name, slug, contact_email, contact_phone,
       status, created_at, updated_at
FROM organizations
WHERE slug = $1
"#;

const LIST_ACTIVE: &str = r#"
SELECT id, name, slug, contact_email, contact_phone,
       status, created_at, updated_at
FROM organizations
WHERE status = 'active'
ORDER BY name
"#;

const LIST_ALL: &str = r#"
SELECT id, name, slug, contact_email, contact_phone,
       status, created_at, updated_at
FROM organizations
ORDER BY name
"#;

#[derive(sqlx::FromRow)]
struct OrgRow {
    id: Uuid,
    name: String,
    slug: String,
    contact_email: String,
    contact_phone: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<OrgRow> for Organization {
    type Error = TenancyError;
    fn try_from(r: OrgRow) -> Result<Self, TenancyError> {
        Ok(Organization::reconstitute(
            OrganizationId::from_uuid(r.id),
            r.name,
            r.slug,
            r.contact_email,
            r.contact_phone,
            OrganizationStatus::from_str(&r.status)?,
            r.created_at,
            r.updated_at,
        ))
    }
}
