use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::RestaurantOperationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TableStatus {
    #[default]
    Free,
    Seated,
    Reserved,
    Dirty,
}

impl TableStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            TableStatus::Free => "free",
            TableStatus::Seated => "seated",
            TableStatus::Reserved => "reserved",
            TableStatus::Dirty => "dirty",
        }
    }
}

impl FromStr for TableStatus {
    type Err = RestaurantOperationsError;
    fn from_str(s: &str) -> Result<Self, RestaurantOperationsError> {
        match s {
            "free" => Ok(TableStatus::Free),
            "seated" => Ok(TableStatus::Seated),
            "reserved" => Ok(TableStatus::Reserved),
            "dirty" => Ok(TableStatus::Dirty),
            other => Err(RestaurantOperationsError::InvalidTableStatus(
                other.to_string(),
            )),
        }
    }
}
