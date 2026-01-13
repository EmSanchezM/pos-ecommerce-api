use crate::health::domain::HealthStatus;

pub struct HealthCheckUseCase {
    service_name: String,
    version: String,
}

impl HealthCheckUseCase {
    pub fn new(service_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            version: version.into(),
        }
    }

    pub fn execute(&self) -> HealthStatus {
        // Aquí se pueden agregar verificaciones adicionales:
        // - Conexión a base de datos
        // - Servicios externos
        // - Estado de memoria/CPU
        HealthStatus::healthy(&self.service_name, &self.version)
    }
}
