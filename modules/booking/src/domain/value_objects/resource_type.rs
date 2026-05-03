use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::BookingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Person,
    Equipment,
    Room,
}

impl ResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            ResourceType::Person => "person",
            ResourceType::Equipment => "equipment",
            ResourceType::Room => "room",
        }
    }
}

impl FromStr for ResourceType {
    type Err = BookingError;
    fn from_str(s: &str) -> Result<Self, BookingError> {
        match s {
            "person" => Ok(ResourceType::Person),
            "equipment" => Ok(ResourceType::Equipment),
            "room" => Ok(ResourceType::Room),
            other => Err(BookingError::InvalidResourceType(other.to_string())),
        }
    }
}
