//! Vendor code generation utilities.
//!
//! This module provides functions to automatically generate vendor codes
//! based on the vendor's legal name.

use unidecode::unidecode;

/// Generates a vendor code prefix from the legal name.
///
/// The prefix is created by:
/// 1. Converting accented characters to ASCII equivalents (é → e, ñ → n, etc.)
/// 2. Converting to uppercase
/// 3. Keeping only alphanumeric characters (removing spaces, punctuation, etc.)
/// 4. Taking the first 8 characters
///
/// # Arguments
/// * `legal_name` - The legal name of the vendor (e.g., "Distribuidora López S.A.")
///
/// # Returns
/// A string prefix of up to 8 uppercase alphanumeric characters (e.g., "DISTRIBU")
///
/// # Examples
/// ```
/// use purchasing::application::utils::generate_vendor_code_prefix;
///
/// assert_eq!(generate_vendor_code_prefix("Distribuidora López S.A."), "DISTRIBU");
/// assert_eq!(generate_vendor_code_prefix("Café Express"), "CAFEEXPR");
/// assert_eq!(generate_vendor_code_prefix("ABC"), "ABC");
/// ```
pub fn generate_vendor_code_prefix(legal_name: &str) -> String {
    // Convert accented characters to ASCII, uppercase, and filter alphanumeric
    let normalized: String = unidecode(legal_name)
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(8)
        .collect();

    // If the result is empty (edge case), return a default
    if normalized.is_empty() {
        return "VENDOR".to_string();
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_name() {
        assert_eq!(generate_vendor_code_prefix("Distribuidora"), "DISTRIBU");
    }

    #[test]
    fn test_name_with_accents() {
        assert_eq!(generate_vendor_code_prefix("Distribuidora López"), "DISTRIBU");
        assert_eq!(generate_vendor_code_prefix("Café Express"), "CAFEEXPR");
        assert_eq!(generate_vendor_code_prefix("Señor Tacos"), "SENORTAC");
    }

    #[test]
    fn test_name_with_special_chars() {
        assert_eq!(generate_vendor_code_prefix("ABC & Co. S.A."), "ABCCOSA");
        assert_eq!(generate_vendor_code_prefix("Test-Company"), "TESTCOMP");
    }

    #[test]
    fn test_short_name() {
        assert_eq!(generate_vendor_code_prefix("ABC"), "ABC");
        assert_eq!(generate_vendor_code_prefix("AB"), "AB");
    }

    #[test]
    fn test_empty_or_special_only() {
        assert_eq!(generate_vendor_code_prefix(""), "VENDOR");
        assert_eq!(generate_vendor_code_prefix("   "), "VENDOR");
        assert_eq!(generate_vendor_code_prefix("---"), "VENDOR");
    }

    #[test]
    fn test_lowercase_conversion() {
        assert_eq!(generate_vendor_code_prefix("lowercase name"), "LOWERCAS");
    }

    #[test]
    fn test_mixed_case() {
        assert_eq!(generate_vendor_code_prefix("MiXeD CaSe"), "MIXEDCAS");
    }

    #[test]
    fn test_numbers_included() {
        assert_eq!(generate_vendor_code_prefix("Company 123"), "COMPANY1");
    }
}
