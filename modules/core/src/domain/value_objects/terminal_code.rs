// TerminalCode value object
// Validates: alphanumeric with hyphens, 3-20 characters, auto-uppercase

use crate::CoreError;
use serde::{Deserialize, Serialize};

/// Terminal code - unique identifier within a store
/// 
/// Validation rules:
/// - Length: 3-20 characters
/// - Allowed characters: alphanumeric (A-Z, a-z, 0-9) and hyphens (-)
/// - Automatically converted to uppercase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCode(String);

impl TerminalCode {
    /// Creates a new TerminalCode with validation
    /// 
    /// # Arguments
    /// * `code` - The terminal code string
    /// 
    /// # Returns
    /// * `Ok(TerminalCode)` - If validation passes
    /// * `Err(CoreError::InvalidTerminalCode)` - If validation fails
    /// 
    /// # Validation Rules
    /// - Must be 3-20 characters long
    /// - Only alphanumeric characters and hyphens allowed
    /// - Automatically converted to uppercase
    pub fn new(code: &str) -> Result<Self, CoreError> {
        // Validate length
        if code.len() < 3 || code.len() > 20 {
            return Err(CoreError::InvalidTerminalCode);
        }

        // Validate characters: alphanumeric and hyphens only
        if !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(CoreError::InvalidTerminalCode);
        }

        // Convert to uppercase and return
        Ok(Self(code.to_uppercase()))
    }

    /// Returns the code as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
