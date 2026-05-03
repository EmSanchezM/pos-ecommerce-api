//! Organization — top-level tenant entity.
//!
//! Slug is the URL-safe identifier (`acme-corp`). It must match
//! `^[a-z0-9][a-z0-9-]{1,58}[a-z0-9]$` — lowercase alphanumeric or hyphen,
//! 3-60 chars, no leading/trailing hyphen. The slug is also enforced by the
//! DB CHECK constraint, but we validate up front to surface a clean error
//! code instead of a 500.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TenancyError;
use crate::domain::value_objects::{OrganizationId, OrganizationStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    slug: String,
    contact_email: String,
    contact_phone: Option<String>,
    status: OrganizationStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn register(
        name: String,
        slug: String,
        contact_email: String,
        contact_phone: Option<String>,
    ) -> Result<Self, TenancyError> {
        if name.trim().is_empty() {
            return Err(TenancyError::Validation("name is required".to_string()));
        }
        validate_slug(&slug)?;
        if contact_email.trim().is_empty() || !contact_email.contains('@') {
            return Err(TenancyError::Validation(
                "contact_email is required and must look like an email".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: OrganizationId::new(),
            name,
            slug,
            contact_email,
            contact_phone,
            status: OrganizationStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: OrganizationId,
        name: String,
        slug: String,
        contact_email: String,
        contact_phone: Option<String>,
        status: OrganizationStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            slug,
            contact_email,
            contact_phone,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn update_contact(
        &mut self,
        name: String,
        contact_email: String,
        contact_phone: Option<String>,
    ) -> Result<(), TenancyError> {
        if name.trim().is_empty() {
            return Err(TenancyError::Validation("name is required".to_string()));
        }
        if !contact_email.contains('@') {
            return Err(TenancyError::Validation(
                "contact_email must look like an email".to_string(),
            ));
        }
        self.name = name;
        self.contact_email = contact_email;
        self.contact_phone = contact_phone;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn transition(&mut self, to: OrganizationStatus) -> Result<(), TenancyError> {
        if !self.status.can_transition_to(to) {
            return Err(TenancyError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<(), TenancyError> {
        self.transition(OrganizationStatus::Suspended)
    }

    pub fn activate(&mut self) -> Result<(), TenancyError> {
        self.transition(OrganizationStatus::Active)
    }

    pub fn id(&self) -> OrganizationId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn slug(&self) -> &str {
        &self.slug
    }
    pub fn contact_email(&self) -> &str {
        &self.contact_email
    }
    pub fn contact_phone(&self) -> Option<&str> {
        self.contact_phone.as_deref()
    }
    pub fn status(&self) -> OrganizationStatus {
        self.status
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

fn validate_slug(slug: &str) -> Result<(), TenancyError> {
    let len = slug.chars().count();
    if !(3..=60).contains(&len) {
        return Err(TenancyError::InvalidSlug(slug.to_string()));
    }
    let first_last_ok = slug
        .chars()
        .next()
        .map(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .unwrap_or(false)
        && slug
            .chars()
            .last()
            .map(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            .unwrap_or(false);
    if !first_last_ok {
        return Err(TenancyError::InvalidSlug(slug.to_string()));
    }
    let body_ok = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !body_ok {
        return Err(TenancyError::InvalidSlug(slug.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_canonical() {
        for s in ["acme", "demo-resto", "a1b2", "tres-palabras-mas"] {
            assert!(validate_slug(s).is_ok(), "expected `{}` to validate", s);
        }
    }

    #[test]
    fn slug_rejects_garbage() {
        for s in [
            "a",               // too short
            "ab",              // too short
            "-leading",        // leading hyphen
            "trailing-",       // trailing hyphen
            "UPPERCASE",       // uppercase
            "with space",      // space
            "with_underscore", // underscore
            "ñoñería",         // non-ascii
            &"a".repeat(61),   // too long
        ] {
            assert!(validate_slug(s).is_err(), "expected `{}` to fail", s);
        }
    }
}
