//! PostgreSQL CustomerRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::{Address, Customer};
use crate::domain::repositories::{CustomerFilter, CustomerRepository};
use crate::domain::value_objects::{CustomerId, CustomerType};
use crate::SalesError;
use identity::{StoreId, UserId};

/// PostgreSQL implementation of CustomerRepository
pub struct PgCustomerRepository {
    pool: PgPool,
}

impl PgCustomerRepository {
    /// Creates a new PgCustomerRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for PgCustomerRepository {
    async fn save(&self, customer: &Customer) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO customers (
                id, store_id, customer_type, code, first_name, last_name, company_name,
                email, phone, tax_id, address_line1, address_line2, address_city,
                address_state, address_postal_code, address_country, user_id, is_active,
                total_purchases, purchase_count, last_purchase_at, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
            "#,
        )
        .bind(customer.id().into_uuid())
        .bind(customer.store_id().into_uuid())
        .bind(customer.customer_type().to_string())
        .bind(customer.code())
        .bind(customer.first_name())
        .bind(customer.last_name())
        .bind(customer.company_name())
        .bind(customer.email())
        .bind(customer.phone())
        .bind(customer.tax_id())
        .bind(customer.billing_address().line1.as_deref())
        .bind(customer.billing_address().line2.as_deref())
        .bind(customer.billing_address().city.as_deref())
        .bind(customer.billing_address().state.as_deref())
        .bind(customer.billing_address().postal_code.as_deref())
        .bind(customer.billing_address().country.as_deref())
        .bind(customer.user_id().map(|u| u.into_uuid()))
        .bind(customer.is_active())
        .bind(customer.total_purchases())
        .bind(customer.purchase_count())
        .bind(customer.last_purchase_at())
        .bind(customer.notes())
        .bind(customer.created_at())
        .bind(customer.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: CustomerId) -> Result<Option<Customer>, SalesError> {
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"
            SELECT id, store_id, customer_type, code, first_name, last_name, company_name,
                   email, phone, tax_id, address_line1, address_line2, address_city,
                   address_state, address_postal_code, address_country, user_id, is_active,
                   total_purchases, purchase_count, last_purchase_at, notes, created_at, updated_at
            FROM customers
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_code(
        &self,
        store_id: StoreId,
        code: &str,
    ) -> Result<Option<Customer>, SalesError> {
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"
            SELECT id, store_id, customer_type, code, first_name, last_name, company_name,
                   email, phone, tax_id, address_line1, address_line2, address_city,
                   address_state, address_postal_code, address_country, user_id, is_active,
                   total_purchases, purchase_count, last_purchase_at, notes, created_at, updated_at
            FROM customers
            WHERE store_id = $1 AND code = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_email(
        &self,
        store_id: StoreId,
        email: &str,
    ) -> Result<Option<Customer>, SalesError> {
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"
            SELECT id, store_id, customer_type, code, first_name, last_name, company_name,
                   email, phone, tax_id, address_line1, address_line2, address_city,
                   address_state, address_postal_code, address_country, user_id, is_active,
                   total_purchases, purchase_count, last_purchase_at, notes, created_at, updated_at
            FROM customers
            WHERE store_id = $1 AND email = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, customer: &Customer) -> Result<(), SalesError> {
        let result = sqlx::query(
            r#"
            UPDATE customers
            SET customer_type = $2, code = $3, first_name = $4, last_name = $5,
                company_name = $6, email = $7, phone = $8, tax_id = $9,
                address_line1 = $10, address_line2 = $11, address_city = $12,
                address_state = $13, address_postal_code = $14, address_country = $15,
                user_id = $16, is_active = $17, total_purchases = $18, purchase_count = $19,
                last_purchase_at = $20, notes = $21, updated_at = $22
            WHERE id = $1
            "#,
        )
        .bind(customer.id().into_uuid())
        .bind(customer.customer_type().to_string())
        .bind(customer.code())
        .bind(customer.first_name())
        .bind(customer.last_name())
        .bind(customer.company_name())
        .bind(customer.email())
        .bind(customer.phone())
        .bind(customer.tax_id())
        .bind(customer.billing_address().line1.as_deref())
        .bind(customer.billing_address().line2.as_deref())
        .bind(customer.billing_address().city.as_deref())
        .bind(customer.billing_address().state.as_deref())
        .bind(customer.billing_address().postal_code.as_deref())
        .bind(customer.billing_address().country.as_deref())
        .bind(customer.user_id().map(|u| u.into_uuid()))
        .bind(customer.is_active())
        .bind(customer.total_purchases())
        .bind(customer.purchase_count())
        .bind(customer.last_purchase_at())
        .bind(customer.notes())
        .bind(customer.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SalesError::CustomerNotFound(customer.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: CustomerFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Customer>, i64), SalesError> {
        let offset = (page - 1) * page_size;

        // Build count query
        let mut count_query = String::from("SELECT COUNT(*) FROM customers WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.is_active.is_some() {
            count_query.push_str(&format!(" AND is_active = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(
                " AND (first_name ILIKE ${} OR last_name ILIKE ${} OR company_name ILIKE ${} OR email ILIKE ${})",
                param_idx, param_idx, param_idx, param_idx
            ));
        }

        // Execute count query
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = filter.store_id {
            count_builder = count_builder.bind(store_id.into_uuid());
        }
        if let Some(active) = filter.is_active {
            count_builder = count_builder.bind(active);
        }
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        // Build data query
        let mut data_query = String::from(
            r#"SELECT id, store_id, customer_type, code, first_name, last_name, company_name,
                   email, phone, tax_id, address_line1, address_line2, address_city,
                   address_state, address_postal_code, address_country, user_id, is_active,
                   total_purchases, purchase_count, last_purchase_at, notes, created_at, updated_at
            FROM customers WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.store_id.is_some() {
            data_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.is_active.is_some() {
            data_query.push_str(&format!(" AND is_active = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            data_query.push_str(&format!(
                " AND (first_name ILIKE ${} OR last_name ILIKE ${} OR company_name ILIKE ${} OR email ILIKE ${})",
                param_idx, param_idx, param_idx, param_idx
            ));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY last_name, first_name LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut data_builder = sqlx::query_as::<_, CustomerRow>(&data_query);
        if let Some(store_id) = filter.store_id {
            data_builder = data_builder.bind(store_id.into_uuid());
        }
        if let Some(active) = filter.is_active {
            data_builder = data_builder.bind(active);
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;
        let customers: Result<Vec<Customer>, SalesError> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((customers?, total_count))
    }

    async fn generate_customer_code(&self, store_id: StoreId) -> Result<String, SalesError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM customers WHERE store_id = $1 AND code LIKE 'CUS-%'"
        )
        .bind(store_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("CUS-{:05}", count.0 + 1))
    }
}

// =============================================================================
// Row types for database mapping
// =============================================================================

#[derive(sqlx::FromRow)]
struct CustomerRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    customer_type: String,
    code: String,
    first_name: String,
    last_name: String,
    company_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    tax_id: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    address_city: Option<String>,
    address_state: Option<String>,
    address_postal_code: Option<String>,
    address_country: Option<String>,
    user_id: Option<uuid::Uuid>,
    is_active: bool,
    total_purchases: rust_decimal::Decimal,
    purchase_count: i32,
    last_purchase_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CustomerRow> for Customer {
    type Error = SalesError;

    fn try_from(row: CustomerRow) -> Result<Self, Self::Error> {
        let customer_type: CustomerType = row
            .customer_type
            .parse()
            .unwrap_or(CustomerType::Individual);

        let address = Address::new(
            row.address_line1,
            row.address_line2,
            row.address_city,
            row.address_state,
            row.address_postal_code,
            row.address_country,
        );

        Ok(Customer::reconstitute(
            CustomerId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            customer_type,
            row.code,
            row.first_name,
            row.last_name,
            row.company_name,
            row.email,
            row.phone,
            row.tax_id,
            address,
            row.user_id.map(UserId::from_uuid),
            row.is_active,
            row.total_purchases,
            row.purchase_count,
            row.last_purchase_at,
            row.notes,
            row.created_at,
            row.updated_at,
        ))
    }
}
