//! OrganizationBranding — one row per organization (PK = FK). Cheaper to
//! upsert than to model id/created_by separately. The hex color fields are
//! validated at the domain layer to match `#RRGGBB`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TenancyError;
use crate::domain::value_objects::{OrganizationId, OrganizationTheme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationBranding {
    organization_id: OrganizationId,
    logo_url: Option<String>,
    favicon_url: Option<String>,
    primary_color: Option<String>,
    secondary_color: Option<String>,
    accent_color: Option<String>,
    theme: OrganizationTheme,
    custom_css: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OrganizationBranding {
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_for_org(
        organization_id: OrganizationId,
        logo_url: Option<String>,
        favicon_url: Option<String>,
        primary_color: Option<String>,
        secondary_color: Option<String>,
        accent_color: Option<String>,
        theme: OrganizationTheme,
        custom_css: Option<String>,
    ) -> Result<Self, TenancyError> {
        validate_color(primary_color.as_deref())?;
        validate_color(secondary_color.as_deref())?;
        validate_color(accent_color.as_deref())?;
        let now = Utc::now();
        Ok(Self {
            organization_id,
            logo_url,
            favicon_url,
            primary_color,
            secondary_color,
            accent_color,
            theme,
            custom_css,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        organization_id: OrganizationId,
        logo_url: Option<String>,
        favicon_url: Option<String>,
        primary_color: Option<String>,
        secondary_color: Option<String>,
        accent_color: Option<String>,
        theme: OrganizationTheme,
        custom_css: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            organization_id,
            logo_url,
            favicon_url,
            primary_color,
            secondary_color,
            accent_color,
            theme,
            custom_css,
            created_at,
            updated_at,
        }
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }
    pub fn logo_url(&self) -> Option<&str> {
        self.logo_url.as_deref()
    }
    pub fn favicon_url(&self) -> Option<&str> {
        self.favicon_url.as_deref()
    }
    pub fn primary_color(&self) -> Option<&str> {
        self.primary_color.as_deref()
    }
    pub fn secondary_color(&self) -> Option<&str> {
        self.secondary_color.as_deref()
    }
    pub fn accent_color(&self) -> Option<&str> {
        self.accent_color.as_deref()
    }
    pub fn theme(&self) -> OrganizationTheme {
        self.theme
    }
    pub fn custom_css(&self) -> Option<&str> {
        self.custom_css.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

fn validate_color(color: Option<&str>) -> Result<(), TenancyError> {
    let Some(c) = color else { return Ok(()) };
    if c.len() != 7 || !c.starts_with('#') || !c[1..].chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(TenancyError::InvalidColor(c.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_validation() {
        assert!(validate_color(None).is_ok());
        assert!(validate_color(Some("#000000")).is_ok());
        assert!(validate_color(Some("#FFFFFF")).is_ok());
        assert!(validate_color(Some("#12abCD")).is_ok());
        assert!(validate_color(Some("000000")).is_err()); // missing #
        assert!(validate_color(Some("#0000")).is_err()); // too short
        assert!(validate_color(Some("#GGGGGG")).is_err()); // not hex
    }
}
