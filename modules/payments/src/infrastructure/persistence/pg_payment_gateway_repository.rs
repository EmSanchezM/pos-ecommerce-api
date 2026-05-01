//! PostgreSQL implementation of PaymentGatewayRepository.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::PaymentsError;
use crate::domain::entities::PaymentGateway;
use crate::domain::repositories::PaymentGatewayRepository;
use crate::domain::value_objects::{GatewayConfig, GatewayType, PaymentGatewayId};
use identity::StoreId;

pub struct PgPaymentGatewayRepository {
    pool: PgPool,
}

impl PgPaymentGatewayRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentGatewayRepository for PgPaymentGatewayRepository {
    async fn save(&self, gateway: &PaymentGateway) -> Result<(), PaymentsError> {
        sqlx::query(
            r#"
            INSERT INTO payment_gateways (
                id, store_id, name, gateway_type, is_active, is_default,
                api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
                supported_methods, supported_currencies, webhook_secret,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(gateway.id().into_uuid())
        .bind(gateway.store_id().into_uuid())
        .bind(gateway.name())
        .bind(gateway.gateway_type().to_string())
        .bind(gateway.is_active())
        .bind(gateway.is_default())
        .bind(gateway.config().api_key_encrypted())
        .bind(gateway.config().secret_key_encrypted())
        .bind(gateway.config().merchant_id())
        .bind(gateway.config().is_sandbox())
        .bind(gateway.supported_methods())
        .bind(gateway.supported_currencies())
        .bind(gateway.webhook_secret())
        .bind(gateway.created_at())
        .bind(gateway.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: PaymentGatewayId,
    ) -> Result<Option<PaymentGateway>, PaymentsError> {
        let row = sqlx::query_as::<_, GatewayRow>(SELECT_GATEWAY_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<PaymentGateway>, PaymentsError> {
        let rows = sqlx::query_as::<_, GatewayRow>(
            r#"
            SELECT id, store_id, name, gateway_type, is_active, is_default,
                   api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
                   supported_methods, supported_currencies, webhook_secret,
                   created_at, updated_at
            FROM payment_gateways
            WHERE store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<PaymentGateway>, PaymentsError> {
        let row = sqlx::query_as::<_, GatewayRow>(
            r#"
            SELECT id, store_id, name, gateway_type, is_active, is_default,
                   api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
                   supported_methods, supported_currencies, webhook_secret,
                   created_at, updated_at
            FROM payment_gateways
            WHERE store_id = $1 AND is_default = true AND is_active = true
            LIMIT 1
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, gateway: &PaymentGateway) -> Result<(), PaymentsError> {
        let result = sqlx::query(
            r#"
            UPDATE payment_gateways
            SET name = $2,
                gateway_type = $3,
                is_active = $4,
                is_default = $5,
                api_key_encrypted = $6,
                secret_key_encrypted = $7,
                merchant_id = $8,
                is_sandbox = $9,
                supported_methods = $10,
                supported_currencies = $11,
                webhook_secret = $12,
                updated_at = $13
            WHERE id = $1
            "#,
        )
        .bind(gateway.id().into_uuid())
        .bind(gateway.name())
        .bind(gateway.gateway_type().to_string())
        .bind(gateway.is_active())
        .bind(gateway.is_default())
        .bind(gateway.config().api_key_encrypted())
        .bind(gateway.config().secret_key_encrypted())
        .bind(gateway.config().merchant_id())
        .bind(gateway.config().is_sandbox())
        .bind(gateway.supported_methods())
        .bind(gateway.supported_currencies())
        .bind(gateway.webhook_secret())
        .bind(gateway.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PaymentsError::GatewayNotFound(gateway.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: PaymentGatewayId) -> Result<(), PaymentsError> {
        let result = sqlx::query("DELETE FROM payment_gateways WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(PaymentsError::GatewayNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: PaymentGatewayId,
    ) -> Result<(), PaymentsError> {
        sqlx::query(
            r#"
            UPDATE payment_gateways
            SET is_default = false, updated_at = NOW()
            WHERE store_id = $1 AND id <> $2 AND is_default = true
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(keep.into_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

const SELECT_GATEWAY_SQL: &str = r#"
SELECT id, store_id, name, gateway_type, is_active, is_default,
       api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
       supported_methods, supported_currencies, webhook_secret,
       created_at, updated_at
FROM payment_gateways
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct GatewayRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    name: String,
    gateway_type: String,
    is_active: bool,
    is_default: bool,
    api_key_encrypted: String,
    secret_key_encrypted: String,
    merchant_id: Option<String>,
    is_sandbox: bool,
    supported_methods: Vec<String>,
    supported_currencies: Vec<String>,
    webhook_secret: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<GatewayRow> for PaymentGateway {
    type Error = PaymentsError;

    fn try_from(row: GatewayRow) -> Result<Self, Self::Error> {
        let gateway_type: GatewayType = row
            .gateway_type
            .parse()
            .map_err(|_| PaymentsError::InvalidGatewayType)?;
        let config = GatewayConfig::new(
            row.api_key_encrypted,
            row.secret_key_encrypted,
            row.merchant_id,
            row.is_sandbox,
        );
        Ok(PaymentGateway::reconstitute(
            PaymentGatewayId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            gateway_type,
            row.is_active,
            row.is_default,
            config,
            row.supported_methods,
            row.supported_currencies,
            row.webhook_secret,
            row.created_at,
            row.updated_at,
        ))
    }
}
