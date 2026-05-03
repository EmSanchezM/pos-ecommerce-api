use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::RestaurantOperationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Course {
    Appetizer,
    #[default]
    Main,
    Dessert,
    Drink,
    Other,
}

impl Course {
    pub fn as_str(self) -> &'static str {
        match self {
            Course::Appetizer => "appetizer",
            Course::Main => "main",
            Course::Dessert => "dessert",
            Course::Drink => "drink",
            Course::Other => "other",
        }
    }
}

impl FromStr for Course {
    type Err = RestaurantOperationsError;
    fn from_str(s: &str) -> Result<Self, RestaurantOperationsError> {
        match s {
            "appetizer" => Ok(Course::Appetizer),
            "main" => Ok(Course::Main),
            "dessert" => Ok(Course::Dessert),
            "drink" => Ok(Course::Drink),
            "other" => Ok(Course::Other),
            other => Err(RestaurantOperationsError::InvalidCourse(other.to_string())),
        }
    }
}
