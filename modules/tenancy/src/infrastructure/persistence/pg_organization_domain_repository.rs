use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::TenancyError;
use crate::domain::entities::OrganizationDomain;
use crate::domain::repositories::OrganizationDomainRepository;
use crate::domain::value_objects::{OrganizationDomainId, OrganizationId};

pub struct PgOrganizationDomainRepository {
    pool: PgPool,
}

impl PgOrganizationDomainRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationDomainRepository for PgOrganizationDomainRepository {
    async fn save(&self, d: &OrganizationDomain) -> Result<(), TenancyError> {
        sqlx::query(
            r#"
            INSERT INTO organization_domains (
                id, organization_id, domain, is_verified, is_primary,
                verification_token, verified_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.organization_id().into_uuid())
        .bind(d.domain())
        .bind(d.is_verified())
        .bind(d.is_primary())
        .bind(d.verification_token())
        .bind(d.verified_at())
        .bind(d.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, d: &OrganizationDomain) -> Result<(), TenancyError> {
        let result = sqlx::query(
            r#"
            UPDATE organization_domains
               SET is_verified        = $2,
                   is_primary         = $3,
                   verification_token = $4,
                   verified_at        = $5
             WHERE id = $1
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.is_verified())
        .bind(d.is_primary())
        .bind(d.verification_token())
        .bind(d.verified_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(TenancyError::DomainNotFound(d.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: OrganizationDomainId) -> Result<(), TenancyError> {
        let result = sqlx::query("DELETE FROM organization_domains WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(TenancyError::DomainNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: OrganizationDomainId,
    ) -> Result<Option<OrganizationDomain>, TenancyError> {
        let row = sqlx::query_as::<_, DomainRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(OrganizationDomain::from))
    }

    async fn find_by_domain(
        &self,
        domain: &str,
    ) -> Result<Option<OrganizationDomain>, TenancyError> {
        let row = sqlx::query_as::<_, DomainRow>(SELECT_BY_DOMAIN)
            .bind(domain)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(OrganizationDomain::from))
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationDomain>, TenancyError> {
        let rows = sqlx::query_as::<_, DomainRow>(LIST_BY_ORG)
            .bind(organization_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(OrganizationDomain::from).collect())
    }

    async fn set_primary(
        &self,
        organization_id: OrganizationId,
        target_id: OrganizationDomainId,
    ) -> Result<(), TenancyError> {
        let mut tx = self.pool.begin().await?;
        // Clear is_primary on every other domain of the org first so the
        // partial unique index never sees two TRUE values during the swap.
        sqlx::query(
            r#"
            UPDATE organization_domains
               SET is_primary = FALSE
             WHERE organization_id = $1 AND id <> $2 AND is_primary = TRUE
            "#,
        )
        .bind(organization_id.into_uuid())
        .bind(target_id.into_uuid())
        .execute(&mut *tx)
        .await?;

        let result = sqlx::query(
            r#"
            UPDATE organization_domains
               SET is_primary = TRUE
             WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(target_id.into_uuid())
        .bind(organization_id.into_uuid())
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(TenancyError::DomainNotFound(target_id.into_uuid()));
        }
        tx.commit().await?;
        Ok(())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, organization_id, domain, is_verified, is_primary,
       verification_token, verified_at, created_at
FROM organization_domains
WHERE id = $1
"#;

const SELECT_BY_DOMAIN: &str = r#"
SELECT id, organization_id, domain, is_verified, is_primary,
       verification_token, verified_at, created_at
FROM organization_domains
WHERE domain = $1
"#;

const LIST_BY_ORG: &str = r#"
SELECT id, organization_id, domain, is_verified, is_primary,
       verification_token, verified_at, created_at
FROM organization_domains
WHERE organization_id = $1
ORDER BY is_primary DESC, created_at
"#;

#[derive(sqlx::FromRow)]
struct DomainRow {
    id: Uuid,
    organization_id: Uuid,
    domain: String,
    is_verified: bool,
    is_primary: bool,
    verification_token: Option<String>,
    verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<DomainRow> for OrganizationDomain {
    fn from(r: DomainRow) -> Self {
        OrganizationDomain::reconstitute(
            OrganizationDomainId::from_uuid(r.id),
            OrganizationId::from_uuid(r.organization_id),
            r.domain,
            r.is_verified,
            r.is_primary,
            r.verification_token,
            r.verified_at,
            r.created_at,
        )
    }
}
