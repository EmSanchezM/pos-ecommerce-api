use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Vehicle,
    Equipment,
    Appliance,
    Electronic,
    Other,
}

impl AssetType {
    pub fn as_str(self) -> &'static str {
        match self {
            AssetType::Vehicle => "vehicle",
            AssetType::Equipment => "equipment",
            AssetType::Appliance => "appliance",
            AssetType::Electronic => "electronic",
            AssetType::Other => "other",
        }
    }
}

impl FromStr for AssetType {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "vehicle" => Ok(AssetType::Vehicle),
            "equipment" => Ok(AssetType::Equipment),
            "appliance" => Ok(AssetType::Appliance),
            "electronic" => Ok(AssetType::Electronic),
            "other" => Ok(AssetType::Other),
            other => Err(ServiceOrdersError::InvalidAssetType(other.to_string())),
        }
    }
}
