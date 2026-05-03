use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::TenancyError;
use crate::domain::entities::OrganizationBranding;
use crate::domain::repositories::OrganizationBrandingRepository;
use crate::domain::value_objects::{OrganizationId, OrganizationTheme};

pub struct PgOrganizationBrandingRepository {
    pool: PgPool,
}

impl PgOrganizationBrandingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationBrandingRepository for PgOrganizationBrandingRepository {
    async fn upsert(&self, b: &OrganizationBranding) -> Result<(), TenancyError> {
        sqlx::query(
            r#"
            INSERT INTO organization_branding (
                organization_id, logo_url, favicon_url,
                primary_color, secondary_color, accent_color,
                theme, custom_css, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id) DO UPDATE SET
                logo_url        = EXCLUDED.logo_url,
                favicon_url     = EXCLUDED.favicon_url,
                primary_color   = EXCLUDED.primary_color,
                secondary_color = EXCLUDED.secondary_color,
                accent_color    = EXCLUDED.accent_color,
                theme           = EXCLUDED.theme,
                custom_css      = EXCLUDED.custom_css,
                updated_at      = EXCLUDED.updated_at
            "#,
        )
        .bind(b.organization_id().into_uuid())
        .bind(b.logo_url())
        .bind(b.favicon_url())
        .bind(b.primary_color())
        .bind(b.secondary_color())
        .bind(b.accent_color())
        .bind(b.theme().as_str())
        .bind(b.custom_css())
        .bind(b.created_at())
        .bind(b.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationBranding>, TenancyError> {
        let row = sqlx::query_as::<_, BrandingRow>(
            r#"
            SELECT organization_id, logo_url, favicon_url,
                   primary_color, secondary_color, accent_color,
                   theme, custom_css, created_at, updated_at
            FROM organization_branding
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(OrganizationBranding::try_from).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct BrandingRow {
    organization_id: Uuid,
    logo_url: Option<String>,
    favicon_url: Option<String>,
    primary_color: Option<String>,
    secondary_color: Option<String>,
    accent_color: Option<String>,
    theme: String,
    custom_css: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BrandingRow> for OrganizationBranding {
    type Error = TenancyError;
    fn try_from(r: BrandingRow) -> Result<Self, TenancyError> {
        Ok(OrganizationBranding::reconstitute(
            OrganizationId::from_uuid(r.organization_id),
            r.logo_url,
            r.favicon_url,
            r.primary_color,
            r.secondary_color,
            r.accent_color,
            OrganizationTheme::from_str(&r.theme)?,
            r.custom_css,
            r.created_at,
            r.updated_at,
        ))
    }
}
