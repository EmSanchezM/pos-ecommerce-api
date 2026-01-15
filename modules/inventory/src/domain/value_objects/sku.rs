// Sku value object - auto-generated stock keeping unit

use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Stock Keeping Unit - auto-generated unique product identifier
/// Format: PRD-{CATEGORY_CODE}-{TIMESTAMP_BASE36}-{RANDOM}
/// Example: PRD-ELC-1A2B3C-X7Y9
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sku(String);

impl Sku {
    /// Generates a new SKU using the format: PRD-{CATEGORY_CODE}-{TIMESTAMP_BASE36}-{RANDOM}
    /// Example: PRD-ELC-1A2B3C-X7Y9
    pub fn generate(category_code: Option<&str>) -> Self {
        let timestamp = Utc::now().timestamp_millis();
        let timestamp_base36 = base36_encode(timestamp as u64);
        let random_suffix = generate_random_suffix();

        let cat_code = category_code
            .map(|c| {
                c.chars()
                    .filter(|ch| ch.is_ascii_alphanumeric())
                    .take(3)
                    .collect::<String>()
                    .to_uppercase()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "GEN".to_string());

        Self(format!("PRD-{}-{}-{}", cat_code, timestamp_base36, random_suffix))
    }

    /// Generates a variant SKU derived from parent SKU
    /// Format: {PARENT_SKU}-V{VARIANT_INDEX}
    pub fn generate_variant(parent_sku: &Sku, variant_index: u32) -> Self {
        Self(format!("{}-V{:03}", parent_sku.0, variant_index))
    }

    /// Reconstitutes a SKU from database (no validation needed, already validated on creation)
    pub fn from_string(value: String) -> Self {
        Self(value)
    }

    /// Returns the SKU as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Sku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Encodes a number in base36 (0-9, A-Z)
fn base36_encode(mut n: u64) -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result = Vec::new();
    
    if n == 0 {
        return "0".to_string();
    }
    
    while n > 0 {
        result.push(CHARS[(n % 36) as usize] as char);
        n /= 36;
    }
    result.reverse();
    result.into_iter().collect()
}

/// Generates a random 4-character alphanumeric suffix
fn generate_random_suffix() -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sku_generate_with_category() {
        let sku = Sku::generate(Some("Electronics"));
        let s = sku.as_str();
        assert!(s.starts_with("PRD-ELE-"));
        assert!(s.len() > 12); // PRD-XXX-timestamp-XXXX
    }

    #[test]
    fn test_sku_generate_without_category() {
        let sku = Sku::generate(None);
        let s = sku.as_str();
        assert!(s.starts_with("PRD-GEN-"));
    }

    #[test]
    fn test_sku_generate_variant() {
        let parent = Sku::from_string("PRD-ELE-ABC123-XY12".to_string());
        let variant = Sku::generate_variant(&parent, 1);
        assert_eq!(variant.as_str(), "PRD-ELE-ABC123-XY12-V001");
    }

    #[test]
    fn test_sku_uniqueness() {
        let sku1 = Sku::generate(Some("Test"));
        let sku2 = Sku::generate(Some("Test"));
        // Due to random suffix, SKUs should be different
        assert_ne!(sku1, sku2);
    }
}
