//! OrganizationDomain — a custom hostname pointing at an organization (e.g.
//! `tienda.acme.com`). v1.0 stores the domain + a verification token but does
//! NOT do the DNS lookup; admins mark `is_verified` manually via the verify
//! endpoint. v1.1 adds an automatic DNS TXT lookup job.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::TenancyError;
use crate::domain::value_objects::{OrganizationDomainId, OrganizationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDomain {
    id: OrganizationDomainId,
    organization_id: OrganizationId,
    domain: String,
    is_verified: bool,
    is_primary: bool,
    verification_token: Option<String>,
    verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl OrganizationDomain {
    pub fn register(organization_id: OrganizationId, domain: String) -> Result<Self, TenancyError> {
        let normalised = normalise_domain(&domain)?;
        // 64-char hex token for the DNS TXT record. Random per registration.
        let verification_token = Uuid::new_v7(Timestamp::now(NoContext)).simple().to_string();
        Ok(Self {
            id: OrganizationDomainId::new(),
            organization_id,
            domain: normalised,
            is_verified: false,
            is_primary: false,
            verification_token: Some(verification_token),
            verified_at: None,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: OrganizationDomainId,
        organization_id: OrganizationId,
        domain: String,
        is_verified: bool,
        is_primary: bool,
        verification_token: Option<String>,
        verified_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            domain,
            is_verified,
            is_primary,
            verification_token,
            verified_at,
            created_at,
        }
    }

    pub fn mark_verified(&mut self) {
        self.is_verified = true;
        self.verified_at = Some(Utc::now());
        // The token has done its job; null it out so it doesn't leak in
        // subsequent reads. v1.1's auto-verifier won't need it either.
        self.verification_token = None;
    }

    pub fn set_primary(&mut self, is_primary: bool) {
        self.is_primary = is_primary;
    }

    pub fn id(&self) -> OrganizationDomainId {
        self.id
    }
    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
    pub fn verification_token(&self) -> Option<&str> {
        self.verification_token.as_deref()
    }
    pub fn verified_at(&self) -> Option<DateTime<Utc>> {
        self.verified_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// Trim, lowercase, and run a permissive shape check on the hostname. We
/// don't try to fully validate per RFC 1035 — Postgres and the unique
/// constraint will catch the leftovers; this just rejects the obvious garbage
/// (empty, longer than 253 chars, contains `://`, contains spaces, etc.).
fn normalise_domain(raw: &str) -> Result<String, TenancyError> {
    let d = raw.trim().to_ascii_lowercase();
    if d.is_empty()
        || d.len() > 253
        || d.contains(' ')
        || d.contains("://")
        || d.contains('/')
        || !d.contains('.')
    {
        return Err(TenancyError::InvalidDomain(raw.to_string()));
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_normalises_and_validates() {
        assert_eq!(
            normalise_domain("  Tienda.Acme.COM ").unwrap(),
            "tienda.acme.com"
        );
        assert!(normalise_domain("not a domain").is_err());
        assert!(normalise_domain("nodot").is_err());
        assert!(normalise_domain("https://acme.com").is_err());
    }
}
