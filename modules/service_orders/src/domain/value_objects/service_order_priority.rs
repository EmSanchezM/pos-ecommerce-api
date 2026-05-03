use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceOrderPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl ServiceOrderPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceOrderPriority::Low => "low",
            ServiceOrderPriority::Normal => "normal",
            ServiceOrderPriority::High => "high",
            ServiceOrderPriority::Urgent => "urgent",
        }
    }
}

impl FromStr for ServiceOrderPriority {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "low" => Ok(ServiceOrderPriority::Low),
            "normal" => Ok(ServiceOrderPriority::Normal),
            "high" => Ok(ServiceOrderPriority::High),
            "urgent" => Ok(ServiceOrderPriority::Urgent),
            other => Err(ServiceOrdersError::InvalidServiceOrderPriority(
                other.to_string(),
            )),
        }
    }
}
