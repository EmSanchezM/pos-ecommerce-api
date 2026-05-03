use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceOrderItemType {
    Labor,
    Part,
}

impl ServiceOrderItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceOrderItemType::Labor => "labor",
            ServiceOrderItemType::Part => "part",
        }
    }
}

impl FromStr for ServiceOrderItemType {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "labor" => Ok(ServiceOrderItemType::Labor),
            "part" => Ok(ServiceOrderItemType::Part),
            other => Err(ServiceOrdersError::InvalidItemType(other.to_string())),
        }
    }
}
