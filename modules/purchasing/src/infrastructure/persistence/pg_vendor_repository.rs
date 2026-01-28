// PostgreSQL VendorRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::Vendor;
use crate::domain::repositories::{VendorFilter, VendorRepository};
use crate::domain::value_objects::VendorId;
use crate::PurchasingError;
use inventory::Currency;

/// PostgreSQL implementation of VendorRepository
pub struct PgVendorRepository {
    pool: PgPool,
}

impl PgVendorRepository {
    /// Creates a new PgVendorRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VendorRepository for PgVendorRepository {
    async fn save(&self, vendor: &Vendor) -> Result<(), PurchasingError> {
        sqlx::query(
            r#"
            INSERT INTO vendors (
                id, code, name, legal_name, tax_id, email, phone, address,
                payment_terms_days, currency, is_active, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(vendor.id().into_uuid())
        .bind(vendor.code())
        .bind(vendor.name())
        .bind(vendor.legal_name())
        .bind(vendor.tax_id())
        .bind(vendor.email())
        .bind(vendor.phone())
        .bind(vendor.address())
        .bind(vendor.payment_terms_days())
        .bind(vendor.currency().as_str())
        .bind(vendor.is_active())
        .bind(vendor.notes())
        .bind(vendor.created_at())
        .bind(vendor.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: VendorId) -> Result<Option<Vendor>, PurchasingError> {
        let row = sqlx::query_as::<_, VendorRow>(
            r#"
            SELECT id, code, name, legal_name, tax_id, email, phone, address,
                   payment_terms_days, currency, is_active, notes, created_at, updated_at
            FROM vendors
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Vendor>, PurchasingError> {
        let row = sqlx::query_as::<_, VendorRow>(
            r#"
            SELECT id, code, name, legal_name, tax_id, email, phone, address,
                   payment_terms_days, currency, is_active, notes, created_at, updated_at
            FROM vendors
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, vendor: &Vendor) -> Result<(), PurchasingError> {
        let result = sqlx::query(
            r#"
            UPDATE vendors
            SET code = $2, name = $3, legal_name = $4, tax_id = $5, email = $6,
                phone = $7, address = $8, payment_terms_days = $9, currency = $10,
                is_active = $11, notes = $12, updated_at = $13
            WHERE id = $1
            "#,
        )
        .bind(vendor.id().into_uuid())
        .bind(vendor.code())
        .bind(vendor.name())
        .bind(vendor.legal_name())
        .bind(vendor.tax_id())
        .bind(vendor.email())
        .bind(vendor.phone())
        .bind(vendor.address())
        .bind(vendor.payment_terms_days())
        .bind(vendor.currency().as_str())
        .bind(vendor.is_active())
        .bind(vendor.notes())
        .bind(vendor.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PurchasingError::VendorNotFound(vendor.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: VendorFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Vendor>, i64), PurchasingError> {
        let offset = (page - 1) * page_size;

        // Build dynamic query for counting
        let mut count_query = String::from("SELECT COUNT(*) FROM vendors WHERE 1=1");
        let mut param_idx = 1;

        if filter.is_active.is_some() {
            count_query.push_str(&format!(" AND is_active = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(
                " AND (name ILIKE ${} OR code ILIKE ${})",
                param_idx,
                param_idx
            ));
        }

        // Execute count query
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(active) = filter.is_active {
            count_builder = count_builder.bind(active);
        }
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        // Build data query
        let mut data_query = String::from(
            r#"SELECT id, code, name, legal_name, tax_id, email, phone, address,
                   payment_terms_days, currency, is_active, notes, created_at, updated_at
            FROM vendors
            WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.is_active.is_some() {
            data_query.push_str(&format!(" AND is_active = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            data_query.push_str(&format!(
                " AND (name ILIKE ${} OR code ILIKE ${})",
                param_idx,
                param_idx
            ));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY name LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        // Execute data query
        let mut data_builder = sqlx::query_as::<_, VendorRow>(&data_query);
        if let Some(active) = filter.is_active {
            data_builder = data_builder.bind(active);
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;

        let vendors: Result<Vec<Vendor>, PurchasingError> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((vendors?, total_count))
    }

    async fn count(&self, filter: VendorFilter) -> Result<i64, PurchasingError> {
        let mut query = String::from("SELECT COUNT(*) FROM vendors WHERE 1=1");
        let mut param_idx = 1;

        if filter.is_active.is_some() {
            query.push_str(&format!(" AND is_active = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            query.push_str(&format!(
                " AND (name ILIKE ${} OR code ILIKE ${})",
                param_idx,
                param_idx
            ));
        }

        let mut builder = sqlx::query_scalar::<_, i64>(&query);
        if let Some(active) = filter.is_active {
            builder = builder.bind(active);
        }
        if let Some(ref search) = filter.search {
            builder = builder.bind(format!("%{}%", search));
        }

        Ok(builder.fetch_one(&self.pool).await?)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, PurchasingError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vendors WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0 > 0)
    }

    async fn exists_by_tax_id(&self, tax_id: &str) -> Result<bool, PurchasingError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vendors WHERE tax_id = $1")
            .bind(tax_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0 > 0)
    }

    async fn exists_by_code_excluding(
        &self,
        code: &str,
        exclude_id: VendorId,
    ) -> Result<bool, PurchasingError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM vendors WHERE code = $1 AND id != $2")
                .bind(code)
                .bind(exclude_id.into_uuid())
                .fetch_one(&self.pool)
                .await?;
        Ok(count.0 > 0)
    }

    async fn exists_by_tax_id_excluding(
        &self,
        tax_id: &str,
        exclude_id: VendorId,
    ) -> Result<bool, PurchasingError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM vendors WHERE tax_id = $1 AND id != $2")
                .bind(tax_id)
                .bind(exclude_id.into_uuid())
                .fetch_one(&self.pool)
                .await?;
        Ok(count.0 > 0)
    }
}

// =============================================================================
// Row types for database mapping
// =============================================================================

/// Internal row type for mapping vendor database results
#[derive(sqlx::FromRow)]
struct VendorRow {
    id: uuid::Uuid,
    code: String,
    name: String,
    legal_name: String,
    tax_id: String,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    payment_terms_days: i32,
    currency: String,
    is_active: bool,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<VendorRow> for Vendor {
    type Error = PurchasingError;

    fn try_from(row: VendorRow) -> Result<Self, Self::Error> {
        let currency = Currency::from_string(row.currency);

        Ok(Vendor::reconstitute(
            VendorId::from_uuid(row.id),
            row.code,
            row.name,
            row.legal_name,
            row.tax_id,
            row.email,
            row.phone,
            row.address,
            row.payment_terms_days,
            currency,
            row.is_active,
            row.notes,
            row.created_at,
            row.updated_at,
        ))
    }
}
