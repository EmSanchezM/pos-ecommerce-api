use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticSeverity::Low => "low",
            DiagnosticSeverity::Medium => "medium",
            DiagnosticSeverity::High => "high",
            DiagnosticSeverity::Critical => "critical",
        }
    }
}

impl FromStr for DiagnosticSeverity {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "low" => Ok(DiagnosticSeverity::Low),
            "medium" => Ok(DiagnosticSeverity::Medium),
            "high" => Ok(DiagnosticSeverity::High),
            "critical" => Ok(DiagnosticSeverity::Critical),
            other => Err(ServiceOrdersError::InvalidDiagnosticSeverity(
                other.to_string(),
            )),
        }
    }
}
