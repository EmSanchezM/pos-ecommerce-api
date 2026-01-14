// CaiNumber value object
// Validates: non-empty, maximum 50 characters

use crate::CoreError;
use serde::{Deserialize, Serialize};

/// CAI (Código de Autorización de Impresión) number
/// 
/// Validation rules:
/// - Must not be empty
/// - Maximum 50 characters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaiNumber(String);

impl CaiNumber {
    /// Creates a new CaiNumber with validation
    /// 
    /// # Arguments
    /// * `cai` - The CAI number string
    /// 
    /// # Returns
    /// * `Ok(CaiNumber)` - If validation passes
    /// * `Err(CoreError::InvalidCaiNumber)` - If validation fails
    /// 
    /// # Validation Rules
    /// - Must not be empty
    /// - Maximum 50 characters
    pub fn new(cai: &str) -> Result<Self, CoreError> {
        // Validate not empty
        if cai.is_empty() {
            return Err(CoreError::InvalidCaiNumber);
        }

        // Validate maximum length
        if cai.len() > 50 {
            return Err(CoreError::InvalidCaiNumber);
        }

        Ok(Self(cai.to_string()))
    }

    /// Returns the CAI number as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
