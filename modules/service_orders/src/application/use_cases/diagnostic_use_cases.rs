use std::sync::Arc;

use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::application::dtos::AddDiagnosticCommand;
use crate::domain::entities::Diagnostic;
use crate::domain::repositories::{DiagnosticRepository, ServiceOrderRepository};
use crate::domain::value_objects::ServiceOrderId;

pub struct AddDiagnosticUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    diagnostics: Arc<dyn DiagnosticRepository>,
}

impl AddDiagnosticUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        diagnostics: Arc<dyn DiagnosticRepository>,
    ) -> Self {
        Self {
            orders,
            diagnostics,
        }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
        technician_user_id: Option<Uuid>,
        cmd: AddDiagnosticCommand,
    ) -> Result<Diagnostic, ServiceOrdersError> {
        let order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;
        if order.status().is_terminal() {
            return Err(ServiceOrdersError::CannotModifyTerminalOrder);
        }
        let diagnostic = Diagnostic::new(
            order_id,
            technician_user_id,
            cmd.findings,
            cmd.recommended_actions,
            cmd.severity,
        )?;
        self.diagnostics.save(&diagnostic).await?;
        Ok(diagnostic)
    }
}

pub struct ListDiagnosticsUseCase {
    diagnostics: Arc<dyn DiagnosticRepository>,
}

impl ListDiagnosticsUseCase {
    pub fn new(diagnostics: Arc<dyn DiagnosticRepository>) -> Self {
        Self { diagnostics }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Diagnostic>, ServiceOrdersError> {
        self.diagnostics.list_by_order(order_id).await
    }
}
