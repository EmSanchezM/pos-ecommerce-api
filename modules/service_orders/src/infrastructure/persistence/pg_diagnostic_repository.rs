use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::Diagnostic;
use crate::domain::repositories::DiagnosticRepository;
use crate::domain::value_objects::{DiagnosticId, DiagnosticSeverity, ServiceOrderId};

pub struct PgDiagnosticRepository {
    pool: PgPool,
}

impl PgDiagnosticRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DiagnosticRepository for PgDiagnosticRepository {
    async fn save(&self, d: &Diagnostic) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            INSERT INTO service_diagnostics (
                id, service_order_id, technician_user_id,
                findings, recommended_actions, severity, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.service_order_id().into_uuid())
        .bind(d.technician_user_id())
        .bind(d.findings())
        .bind(d.recommended_actions())
        .bind(d.severity().as_str())
        .bind(d.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Diagnostic>, ServiceOrdersError> {
        let rows = sqlx::query_as::<_, DiagnosticRow>(
            r#"
            SELECT id, service_order_id, technician_user_id,
                   findings, recommended_actions, severity, created_at
            FROM service_diagnostics
            WHERE service_order_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(order_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Diagnostic::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct DiagnosticRow {
    id: Uuid,
    service_order_id: Uuid,
    technician_user_id: Option<Uuid>,
    findings: String,
    recommended_actions: Option<String>,
    severity: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<DiagnosticRow> for Diagnostic {
    type Error = ServiceOrdersError;
    fn try_from(r: DiagnosticRow) -> Result<Self, ServiceOrdersError> {
        Ok(Diagnostic::reconstitute(
            DiagnosticId::from_uuid(r.id),
            ServiceOrderId::from_uuid(r.service_order_id),
            r.technician_user_id,
            r.findings,
            r.recommended_actions,
            DiagnosticSeverity::from_str(&r.severity)?,
            r.created_at,
        ))
    }
}
