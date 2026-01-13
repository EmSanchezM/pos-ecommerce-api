use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub service_name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn healthy(service_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            status: HealthState::Healthy,
            service_name: service_name.into(),
            version: version.into(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == HealthState::Healthy
    }
}
