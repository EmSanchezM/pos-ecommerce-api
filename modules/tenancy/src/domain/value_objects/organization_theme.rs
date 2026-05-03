use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::TenancyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationTheme {
    Light,
    Dark,
    #[default]
    System,
}

impl OrganizationTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            OrganizationTheme::Light => "light",
            OrganizationTheme::Dark => "dark",
            OrganizationTheme::System => "system",
        }
    }
}

impl FromStr for OrganizationTheme {
    type Err = TenancyError;
    fn from_str(s: &str) -> Result<Self, TenancyError> {
        match s {
            "light" => Ok(OrganizationTheme::Light),
            "dark" => Ok(OrganizationTheme::Dark),
            "system" => Ok(OrganizationTheme::System),
            other => Err(TenancyError::InvalidTheme(other.to_string())),
        }
    }
}
