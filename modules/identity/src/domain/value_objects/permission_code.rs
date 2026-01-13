// PermissionCode value object - validated permission string with format module:action

use crate::IdentityError;
use serde::{Deserialize, Serialize};

/// Validated permission code following the format `module:action`
/// 
/// Examples: `sales:create_invoice`, `inventory:view_stock`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionCode(String);

impl PermissionCode {
    /// Creates a new PermissionCode after validating the format.
    /// 
    /// The code must contain exactly one colon separator with non-empty
    /// parts on both sides (module and action).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use identity::domain::value_objects::PermissionCode;
    /// 
    /// let code = PermissionCode::new("sales:create_invoice").unwrap();
    /// assert_eq!(code.module(), "sales");
    /// assert_eq!(code.action(), "create_invoice");
    /// ```
    pub fn new(code: &str) -> Result<Self, IdentityError> {
        let parts: Vec<&str> = code.split(':').collect();
        
        // Must have exactly two parts (one colon)
        if parts.len() != 2 {
            return Err(IdentityError::InvalidPermissionFormat);
        }
        
        let module = parts[0];
        let action = parts[1];
        
        // Both parts must be non-empty
        if module.is_empty() || action.is_empty() {
            return Err(IdentityError::InvalidPermissionFormat);
        }
        
        Ok(Self(code.to_string()))
    }

    /// Returns the module part of the permission code (before the colon)
    pub fn module(&self) -> &str {
        self.0.split(':').next().unwrap()
    }

    /// Returns the action part of the permission code (after the colon)
    pub fn action(&self) -> &str {
        self.0.split(':').nth(1).unwrap()
    }

    /// Returns the full permission code as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PermissionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PermissionCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_permission_code() {
        let code = PermissionCode::new("sales:create_invoice").unwrap();
        assert_eq!(code.module(), "sales");
        assert_eq!(code.action(), "create_invoice");
        assert_eq!(code.as_str(), "sales:create_invoice");
    }

    #[test]
    fn test_permission_code_with_underscores() {
        let code = PermissionCode::new("inventory:view_stock_levels").unwrap();
        assert_eq!(code.module(), "inventory");
        assert_eq!(code.action(), "view_stock_levels");
    }

    #[test]
    fn test_invalid_no_colon() {
        let result = PermissionCode::new("salescreateinvoice");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_invalid_multiple_colons() {
        let result = PermissionCode::new("sales:create:invoice");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_invalid_empty_module() {
        let result = PermissionCode::new(":create_invoice");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_invalid_empty_action() {
        let result = PermissionCode::new("sales:");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_invalid_empty_string() {
        let result = PermissionCode::new("");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_invalid_only_colon() {
        let result = PermissionCode::new(":");
        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[test]
    fn test_permission_code_equality() {
        let code1 = PermissionCode::new("sales:create").unwrap();
        let code2 = PermissionCode::new("sales:create").unwrap();
        let code3 = PermissionCode::new("sales:delete").unwrap();
        
        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_permission_code_display() {
        let code = PermissionCode::new("sales:create").unwrap();
        assert_eq!(format!("{}", code), "sales:create");
    }

    #[test]
    fn test_permission_code_hash() {
        use std::collections::HashSet;
        
        let code1 = PermissionCode::new("sales:create").unwrap();
        let code2 = PermissionCode::new("sales:create").unwrap();
        
        let mut set = HashSet::new();
        set.insert(code1);
        set.insert(code2);
        
        assert_eq!(set.len(), 1);
    }
}
