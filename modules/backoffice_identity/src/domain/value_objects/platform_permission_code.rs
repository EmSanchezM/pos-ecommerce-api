use crate::error::BackofficeIdentityError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static PLATFORM_PERM_REGEX: OnceLock<Regex> = OnceLock::new();

fn platform_perm_regex() -> &'static Regex {
    PLATFORM_PERM_REGEX.get_or_init(|| {
        Regex::new(r"^platform:[a-z]+\.[a-z_]+$").expect("valid regex")
    })
}

/// Validated platform permission code following the format `platform:resource.action`.
/// Examples: `platform:org.list`, `platform:user.impersonate`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlatformPermissionCode(String);

impl PlatformPermissionCode {
    pub fn new(code: &str) -> Result<Self, BackofficeIdentityError> {
        if platform_perm_regex().is_match(code) {
            Ok(Self(code.to_string()))
        } else {
            Err(BackofficeIdentityError::InvalidPermissionCodeFormat)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn resource(&self) -> &str {
        let after_colon = self.0.split(':').nth(1).unwrap();
        after_colon.split('.').next().unwrap()
    }

    pub fn action(&self) -> &str {
        let after_colon = self.0.split(':').nth(1).unwrap();
        after_colon.split('.').nth(1).unwrap()
    }
}

impl std::fmt::Display for PlatformPermissionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PlatformPermissionCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_platform_permission() {
        let code = PlatformPermissionCode::new("platform:org.list").unwrap();
        assert_eq!(code.as_str(), "platform:org.list");
        assert_eq!(code.resource(), "org");
        assert_eq!(code.action(), "list");
    }

    #[test]
    fn test_valid_with_underscore_in_action() {
        let code = PlatformPermissionCode::new("platform:subscription.force_cancel").unwrap();
        assert_eq!(code.resource(), "subscription");
        assert_eq!(code.action(), "force_cancel");
    }

    #[test]
    fn test_all_thirteen_permissions_valid() {
        let perms = [
            "platform:org.list",
            "platform:org.suspend",
            "platform:org.create",
            "platform:org.update",
            "platform:plan.create",
            "platform:plan.update",
            "platform:plan.read",
            "platform:subscription.force_cancel",
            "platform:subscription.override_billing",
            "platform:dunning.trigger",
            "platform:audit.read",
            "platform:user.impersonate",
            "platform:analytics.read",
        ];
        for p in &perms {
            assert!(PlatformPermissionCode::new(p).is_ok(), "failed: {}", p);
        }
    }

    #[test]
    fn test_invalid_wrong_prefix() {
        assert!(PlatformPermissionCode::new("system:admin").is_err());
    }

    #[test]
    fn test_invalid_no_dot_separator() {
        assert!(PlatformPermissionCode::new("platform:orglist").is_err());
    }

    #[test]
    fn test_invalid_uppercase_in_resource() {
        assert!(PlatformPermissionCode::new("platform:Org.list").is_err());
    }

    #[test]
    fn test_invalid_empty_resource() {
        assert!(PlatformPermissionCode::new("platform:.list").is_err());
    }

    #[test]
    fn test_invalid_empty_action() {
        assert!(PlatformPermissionCode::new("platform:org.").is_err());
    }

    #[test]
    fn test_invalid_wrong_format() {
        assert!(PlatformPermissionCode::new("org.list").is_err());
    }

    #[test]
    fn test_equality() {
        let c1 = PlatformPermissionCode::new("platform:org.list").unwrap();
        let c2 = PlatformPermissionCode::new("platform:org.list").unwrap();
        assert_eq!(c1, c2);
    }
}
