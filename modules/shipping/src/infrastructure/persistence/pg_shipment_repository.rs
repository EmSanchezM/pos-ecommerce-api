use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::ShippingError;
use crate::domain::entities::Shipment;
use crate::domain::repositories::{ShipmentFilter, ShipmentRepository};
use crate::domain::value_objects::{
    DeliveryProviderId, DriverId, ShipmentId, ShipmentStatus, ShippingMethodId, ShippingMethodType,
};
use identity::StoreId;
use sales::SaleId;

pub struct PgShipmentRepository {
    pool: PgPool,
}

impl PgShipmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    s.id, s.store_id, s.sale_id, s.shipping_method_id, m.method_type as method_type,
    s.driver_id, s.delivery_provider_id,
    s.pickup_code, s.pickup_ready_at, s.pickup_expires_at,
    s.picked_up_at, s.picked_up_by_name,
    s.requires_cash_collection, s.cash_amount,
    s.status, s.tracking_number, s.carrier_name,
    s.shipping_cost, s.currency, s.weight_kg,
    s.recipient_name, s.recipient_phone,
    s.address_line1, s.address_line2,
    s.city, s.state, s.postal_code, s.country,
    s.notes, s.failure_reason, s.attempt_count,
    s.shipped_at, s.delivered_at, s.estimated_delivery,
    s.cancelled_at, s.cancel_reason,
    s.created_at, s.updated_at
"#;

#[async_trait]
impl ShipmentRepository for PgShipmentRepository {
    async fn save(&self, s: &Shipment) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO shipments
              (id, store_id, sale_id, shipping_method_id,
               driver_id, delivery_provider_id,
               pickup_code, pickup_ready_at, pickup_expires_at,
               picked_up_at, picked_up_by_name,
               requires_cash_collection, cash_amount,
               status, tracking_number, carrier_name,
               shipping_cost, currency, weight_kg,
               recipient_name, recipient_phone,
               address_line1, address_line2, city, state, postal_code, country,
               notes, failure_reason, attempt_count,
               shipped_at, delivered_at, estimated_delivery,
               cancelled_at, cancel_reason,
               created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,
                      $17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,
                      $31,$32,$33,$34,$35,$36,$37)"#,
        )
        .bind(s.id().into_uuid())
        .bind(s.store_id().into_uuid())
        .bind(s.sale_id().into_uuid())
        .bind(s.shipping_method_id().into_uuid())
        .bind(s.driver_id().map(|d| d.into_uuid()))
        .bind(s.delivery_provider_id().map(|p| p.into_uuid()))
        .bind(s.pickup_code())
        .bind(s.pickup_ready_at())
        .bind(s.pickup_expires_at())
        .bind(s.picked_up_at())
        .bind(s.picked_up_by_name())
        .bind(s.requires_cash_collection())
        .bind(s.cash_amount())
        .bind(s.status().to_string())
        .bind(s.tracking_number())
        .bind(s.carrier_name())
        .bind(s.shipping_cost())
        .bind(s.currency())
        .bind(s.weight_kg())
        .bind(s.recipient_name())
        .bind(s.recipient_phone())
        .bind(s.address_line1())
        .bind(s.address_line2())
        .bind(s.city())
        .bind(s.state())
        .bind(s.postal_code())
        .bind(s.country())
        .bind(s.notes())
        .bind(s.failure_reason())
        .bind(s.attempt_count())
        .bind(s.shipped_at())
        .bind(s.delivered_at())
        .bind(s.estimated_delivery())
        .bind(s.cancelled_at())
        .bind(s.cancel_reason())
        .bind(s.created_at())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: ShipmentId) -> Result<Option<Shipment>, ShippingError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s
             JOIN shipping_methods m ON m.id = s.shipping_method_id
             WHERE s.id = $1 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ShipmentRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Option<Shipment>, ShippingError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s
             JOIN shipping_methods m ON m.id = s.shipping_method_id
             WHERE s.sale_id = $1 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ShipmentRow>(&sql)
            .bind(sale_id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_tracking(
        &self,
        tracking_number: &str,
    ) -> Result<Option<Shipment>, ShippingError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s
             JOIN shipping_methods m ON m.id = s.shipping_method_id
             WHERE s.tracking_number = $1 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ShipmentRow>(&sql)
            .bind(tracking_number)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_pickup_code(
        &self,
        store_id: StoreId,
        pickup_code: &str,
    ) -> Result<Option<Shipment>, ShippingError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s
             JOIN shipping_methods m ON m.id = s.shipping_method_id
             WHERE s.store_id = $1 AND s.pickup_code = $2 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ShipmentRow>(&sql)
            .bind(store_id.into_uuid())
            .bind(pickup_code)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, s: &Shipment) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE shipments SET
                 status=$2,
                 driver_id=$3, delivery_provider_id=$4,
                 pickup_code=$5, pickup_ready_at=$6, pickup_expires_at=$7,
                 picked_up_at=$8, picked_up_by_name=$9,
                 tracking_number=$10, carrier_name=$11,
                 notes=$12, failure_reason=$13, attempt_count=$14,
                 shipped_at=$15, delivered_at=$16, estimated_delivery=$17,
                 cancelled_at=$18, cancel_reason=$19,
                 updated_at=$20
               WHERE id=$1"#,
        )
        .bind(s.id().into_uuid())
        .bind(s.status().to_string())
        .bind(s.driver_id().map(|d| d.into_uuid()))
        .bind(s.delivery_provider_id().map(|p| p.into_uuid()))
        .bind(s.pickup_code())
        .bind(s.pickup_ready_at())
        .bind(s.pickup_expires_at())
        .bind(s.picked_up_at())
        .bind(s.picked_up_by_name())
        .bind(s.tracking_number())
        .bind(s.carrier_name())
        .bind(s.notes())
        .bind(s.failure_reason())
        .bind(s.attempt_count())
        .bind(s.shipped_at())
        .bind(s.delivered_at())
        .bind(s.estimated_delivery())
        .bind(s.cancelled_at())
        .bind(s.cancel_reason())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShipmentNotFound(s.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: ShipmentFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Shipment>, i64), ShippingError> {
        let offset = (page - 1) * page_size;

        let mut data = QueryBuilder::<Postgres>::new(format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s JOIN shipping_methods m ON m.id = s.shipping_method_id WHERE 1=1"
        ));
        let mut count = QueryBuilder::<Postgres>::new(
            "SELECT COUNT(*) FROM shipments s JOIN shipping_methods m ON m.id = s.shipping_method_id WHERE 1=1",
        );

        push_filters(&mut data, &filter);
        push_filters(&mut count, &filter);

        data.push(" ORDER BY s.created_at DESC LIMIT ");
        data.push_bind(page_size);
        data.push(" OFFSET ");
        data.push_bind(offset);

        let rows: Vec<ShipmentRow> = data
            .build_query_as::<ShipmentRow>()
            .fetch_all(&self.pool)
            .await?;
        let total: (i64,) = count
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await?;

        let items: Result<Vec<Shipment>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        Ok((items?, total.0))
    }

    async fn find_expired_pickups(&self) -> Result<Vec<Shipment>, ShippingError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM shipments s
             JOIN shipping_methods m ON m.id = s.shipping_method_id
             WHERE s.status = 'ready_for_pickup'
               AND s.pickup_expires_at IS NOT NULL
               AND s.pickup_expires_at < NOW()"
        );
        let rows = sqlx::query_as::<_, ShipmentRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

fn push_filters<'q>(qb: &mut QueryBuilder<'q, Postgres>, f: &'q ShipmentFilter) {
    if let Some(store_id) = f.store_id {
        qb.push(" AND s.store_id = ");
        qb.push_bind(store_id.into_uuid());
    }
    if let Some(sale_id) = f.sale_id {
        qb.push(" AND s.sale_id = ");
        qb.push_bind(sale_id.into_uuid());
    }
    if let Some(status) = f.status {
        qb.push(" AND s.status = ");
        qb.push_bind(status.to_string());
    }
    if let Some(method_type) = f.method_type {
        qb.push(" AND m.method_type = ");
        qb.push_bind(method_type.to_string());
    }
    if let Some(driver_id) = f.driver_id {
        qb.push(" AND s.driver_id = ");
        qb.push_bind(driver_id.into_uuid());
    }
    if let Some(date_from) = f.date_from {
        qb.push(" AND s.created_at >= ");
        qb.push_bind(date_from);
    }
    if let Some(date_to) = f.date_to {
        qb.push(" AND s.created_at <= ");
        qb.push_bind(date_to);
    }
    if let Some(search) = &f.search {
        let p = format!("%{}%", search);
        qb.push(" AND (s.tracking_number ILIKE ");
        qb.push_bind(p.clone());
        qb.push(" OR s.recipient_name ILIKE ");
        qb.push_bind(p.clone());
        qb.push(" OR s.pickup_code ILIKE ");
        qb.push_bind(p);
        qb.push(")");
    }
}

#[derive(sqlx::FromRow)]
struct ShipmentRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    sale_id: uuid::Uuid,
    shipping_method_id: uuid::Uuid,
    method_type: String,
    driver_id: Option<uuid::Uuid>,
    delivery_provider_id: Option<uuid::Uuid>,
    pickup_code: Option<String>,
    pickup_ready_at: Option<chrono::DateTime<chrono::Utc>>,
    pickup_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    picked_up_at: Option<chrono::DateTime<chrono::Utc>>,
    picked_up_by_name: Option<String>,
    requires_cash_collection: bool,
    cash_amount: Option<rust_decimal::Decimal>,
    status: String,
    tracking_number: Option<String>,
    carrier_name: Option<String>,
    shipping_cost: rust_decimal::Decimal,
    currency: String,
    weight_kg: Option<rust_decimal::Decimal>,
    recipient_name: String,
    recipient_phone: Option<String>,
    address_line1: String,
    address_line2: Option<String>,
    city: String,
    state: String,
    postal_code: Option<String>,
    country: String,
    notes: Option<String>,
    failure_reason: Option<String>,
    attempt_count: i32,
    shipped_at: Option<chrono::DateTime<chrono::Utc>>,
    delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    estimated_delivery: Option<chrono::DateTime<chrono::Utc>>,
    cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    cancel_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ShipmentRow> for Shipment {
    type Error = ShippingError;
    fn try_from(row: ShipmentRow) -> Result<Self, Self::Error> {
        let method_type: ShippingMethodType = row
            .method_type
            .parse()
            .map_err(|_| ShippingError::InvalidMethodType)?;
        let status: ShipmentStatus = row
            .status
            .parse()
            .map_err(|_| ShippingError::InvalidShipmentStatus)?;
        Ok(Shipment::reconstitute(
            ShipmentId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            SaleId::from_uuid(row.sale_id),
            ShippingMethodId::from_uuid(row.shipping_method_id),
            method_type,
            row.driver_id.map(DriverId::from_uuid),
            row.delivery_provider_id.map(DeliveryProviderId::from_uuid),
            row.pickup_code,
            row.pickup_ready_at,
            row.pickup_expires_at,
            row.picked_up_at,
            row.picked_up_by_name,
            row.requires_cash_collection,
            row.cash_amount,
            status,
            row.tracking_number,
            row.carrier_name,
            row.shipping_cost,
            row.currency,
            row.weight_kg,
            row.recipient_name,
            row.recipient_phone,
            row.address_line1,
            row.address_line2,
            row.city,
            row.state,
            row.postal_code,
            row.country,
            row.notes,
            row.failure_reason,
            row.attempt_count,
            row.shipped_at,
            row.delivered_at,
            row.estimated_delivery,
            row.cancelled_at,
            row.cancel_reason,
            row.created_at,
            row.updated_at,
        ))
    }
}
