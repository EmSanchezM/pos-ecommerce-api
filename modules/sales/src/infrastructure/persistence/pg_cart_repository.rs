//! PostgreSQL CartRepository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;

use crate::domain::entities::{Cart, CartItem};
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::{CartId, CartItemId, CustomerId};
use crate::SalesError;
use identity::StoreId;
use inventory::{Currency, ProductId, ReservationId, UnitOfMeasure, VariantId};

/// PostgreSQL implementation of CartRepository
pub struct PgCartRepository {
    pool: PgPool,
}

impl PgCartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_items(&self, cart_id: CartId) -> Result<Vec<CartItem>, SalesError> {
        let rows = sqlx::query_as::<_, CartItemRow>(
            r#"
            SELECT id, cart_id, product_id, variant_id, sku, name, quantity, unit_of_measure,
                   unit_price, discount_percent, discount_amount, tax_rate, tax_amount,
                   subtotal, total, reservation_id, image_url, created_at, updated_at
            FROM cart_items
            WHERE cart_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(cart_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[async_trait]
impl CartRepository for PgCartRepository {
    async fn save(&self, cart: &Cart) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO carts (
                id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(cart.id().into_uuid())
        .bind(cart.store_id().into_uuid())
        .bind(cart.customer_id().map(|c| c.into_uuid()))
        .bind(cart.session_id())
        .bind(cart.currency().as_str())
        .bind(cart.subtotal())
        .bind(cart.discount_amount())
        .bind(cart.tax_amount())
        .bind(cart.total())
        .bind(cart.item_count())
        .bind(cart.expires_at())
        .bind(cart.converted_to_sale())
        .bind(cart.notes())
        .bind(cart.created_at())
        .bind(cart.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: CartId) -> Result<Option<Cart>, SalesError> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                   tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                   created_at, updated_at
            FROM carts
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_cart(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_with_items(&self, id: CartId) -> Result<Option<Cart>, SalesError> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                   tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                   created_at, updated_at
            FROM carts
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let items = self.load_items(id).await?;
                Ok(Some(r.into_cart(items)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_session(
        &self,
        store_id: StoreId,
        session_id: &str,
    ) -> Result<Option<Cart>, SalesError> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                   tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                   created_at, updated_at
            FROM carts
            WHERE store_id = $1 AND session_id = $2 AND converted_to_sale = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let items = self.load_items(CartId::from_uuid(r.id)).await?;
                Ok(Some(r.into_cart(items)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_customer(
        &self,
        store_id: StoreId,
        customer_id: CustomerId,
    ) -> Result<Option<Cart>, SalesError> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                   tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                   created_at, updated_at
            FROM carts
            WHERE store_id = $1 AND customer_id = $2 AND converted_to_sale = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(customer_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let items = self.load_items(CartId::from_uuid(r.id)).await?;
                Ok(Some(r.into_cart(items)?))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, cart: &Cart) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE carts
            SET customer_id = $2, subtotal = $3, discount_amount = $4, tax_amount = $5,
                total = $6, item_count = $7, expires_at = $8, converted_to_sale = $9,
                notes = $10, updated_at = $11
            WHERE id = $1
            "#,
        )
        .bind(cart.id().into_uuid())
        .bind(cart.customer_id().map(|c| c.into_uuid()))
        .bind(cart.subtotal())
        .bind(cart.discount_amount())
        .bind(cart.tax_amount())
        .bind(cart.total())
        .bind(cart.item_count())
        .bind(cart.expires_at())
        .bind(cart.converted_to_sale())
        .bind(cart.notes())
        .bind(cart.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: CartId) -> Result<(), SalesError> {
        sqlx::query("DELETE FROM carts WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_expired(&self, before: DateTime<Utc>) -> Result<Vec<Cart>, SalesError> {
        let rows = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, store_id, customer_id, session_id, currency, subtotal, discount_amount,
                   tax_amount, total, item_count, expires_at, converted_to_sale, notes,
                   created_at, updated_at
            FROM carts
            WHERE expires_at < $1 AND converted_to_sale = FALSE
            "#,
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_cart(Vec::new()))
            .collect()
    }

    async fn delete_expired(&self, before: DateTime<Utc>) -> Result<i64, SalesError> {
        let result =
            sqlx::query("DELETE FROM carts WHERE expires_at < $1 AND converted_to_sale = FALSE")
                .bind(before)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() as i64)
    }

    async fn save_item(&self, item: &CartItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO cart_items (
                id, cart_id, product_id, variant_id, sku, name, quantity, unit_of_measure,
                unit_price, discount_percent, discount_amount, tax_rate, tax_amount,
                subtotal, total, reservation_id, image_url, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.cart_id().into_uuid())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.sku())
        .bind(item.name())
        .bind(item.quantity())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_price())
        .bind(item.discount_percent())
        .bind(item.discount_amount())
        .bind(item.tax_rate())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.reservation_id().map(|r| r.into_uuid()))
        .bind(item.image_url())
        .bind(item.created_at())
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_item(&self, item: &CartItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE cart_items
            SET quantity = $2, unit_price = $3, discount_percent = $4, discount_amount = $5,
                tax_amount = $6, subtotal = $7, total = $8, reservation_id = $9, updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.quantity())
        .bind(item.unit_price())
        .bind(item.discount_percent())
        .bind(item.discount_amount())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.reservation_id().map(|r| r.into_uuid()))
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_item(&self, item_id: CartItemId) -> Result<(), SalesError> {
        sqlx::query("DELETE FROM cart_items WHERE id = $1")
            .bind(item_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_items_by_cart(&self, cart_id: CartId) -> Result<Vec<CartItem>, SalesError> {
        self.load_items(cart_id).await
    }

    async fn find_item_by_id(&self, item_id: CartItemId) -> Result<Option<CartItem>, SalesError> {
        let row = sqlx::query_as::<_, CartItemRow>(
            r#"
            SELECT id, cart_id, product_id, variant_id, sku, name, quantity, unit_of_measure,
                   unit_price, discount_percent, discount_amount, tax_rate, tax_amount,
                   subtotal, total, reservation_id, image_url, created_at, updated_at
            FROM cart_items
            WHERE id = $1
            "#,
        )
        .bind(item_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn delete_items_by_cart(&self, cart_id: CartId) -> Result<(), SalesError> {
        sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
            .bind(cart_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct CartRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    customer_id: Option<uuid::Uuid>,
    session_id: Option<String>,
    currency: String,
    subtotal: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    item_count: i32,
    expires_at: chrono::DateTime<chrono::Utc>,
    converted_to_sale: bool,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CartRow {
    fn into_cart(self, items: Vec<CartItem>) -> Result<Cart, SalesError> {
        Ok(Cart::reconstitute(
            CartId::from_uuid(self.id),
            StoreId::from_uuid(self.store_id),
            self.customer_id.map(CustomerId::from_uuid),
            self.session_id,
            Currency::from_string(self.currency),
            self.subtotal,
            self.discount_amount,
            self.tax_amount,
            self.total,
            self.item_count,
            items,
            self.expires_at,
            self.converted_to_sale,
            self.notes,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct CartItemRow {
    id: uuid::Uuid,
    cart_id: uuid::Uuid,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    sku: String,
    name: String,
    quantity: rust_decimal::Decimal,
    unit_of_measure: String,
    unit_price: rust_decimal::Decimal,
    discount_percent: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    tax_rate: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    subtotal: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    reservation_id: Option<uuid::Uuid>,
    image_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CartItemRow> for CartItem {
    type Error = SalesError;

    fn try_from(row: CartItemRow) -> Result<Self, Self::Error> {
        let uom = UnitOfMeasure::from_str(&row.unit_of_measure)
            .map_err(|_| SalesError::InvalidUnitOfMeasure)?;

        Ok(CartItem::reconstitute(
            CartItemId::from_uuid(row.id),
            CartId::from_uuid(row.cart_id),
            ProductId::from_uuid(row.product_id),
            row.variant_id.map(VariantId::from_uuid),
            row.sku,
            row.name,
            row.quantity,
            uom,
            row.unit_price,
            row.discount_percent,
            row.discount_amount,
            row.tax_rate,
            row.tax_amount,
            row.subtotal,
            row.total,
            row.reservation_id.map(ReservationId::from_uuid),
            row.image_url,
            row.created_at,
            row.updated_at,
        ))
    }
}
