use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::value_objects::{DiagnosticId, DiagnosticSeverity, ServiceOrderId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    id: DiagnosticId,
    service_order_id: ServiceOrderId,
    technician_user_id: Option<Uuid>,
    findings: String,
    recommended_actions: Option<String>,
    severity: DiagnosticSeverity,
    created_at: DateTime<Utc>,
}

impl Diagnostic {
    pub fn new(
        service_order_id: ServiceOrderId,
        technician_user_id: Option<Uuid>,
        findings: String,
        recommended_actions: Option<String>,
        severity: DiagnosticSeverity,
    ) -> Result<Self, ServiceOrdersError> {
        if findings.trim().is_empty() {
            return Err(ServiceOrdersError::Validation(
                "findings is required".to_string(),
            ));
        }
        Ok(Self {
            id: DiagnosticId::new(),
            service_order_id,
            technician_user_id,
            findings,
            recommended_actions,
            severity,
            created_at: Utc::now(),
        })
    }

    pub fn reconstitute(
        id: DiagnosticId,
        service_order_id: ServiceOrderId,
        technician_user_id: Option<Uuid>,
        findings: String,
        recommended_actions: Option<String>,
        severity: DiagnosticSeverity,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            service_order_id,
            technician_user_id,
            findings,
            recommended_actions,
            severity,
            created_at,
        }
    }

    pub fn id(&self) -> DiagnosticId {
        self.id
    }
    pub fn service_order_id(&self) -> ServiceOrderId {
        self.service_order_id
    }
    pub fn technician_user_id(&self) -> Option<Uuid> {
        self.technician_user_id
    }
    pub fn findings(&self) -> &str {
        &self.findings
    }
    pub fn recommended_actions(&self) -> Option<&str> {
        self.recommended_actions.as_deref()
    }
    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
