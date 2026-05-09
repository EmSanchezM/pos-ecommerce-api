//! `PlanCode` — stable, URL-safe identifier for a `SubscriptionPlan`.
//!
//! Pattern: `^[a-z0-9_]{3,32}$`. Validated up-front so wrong codes surface a
//! clean `InvalidPlanCode` error instead of a `23514` from a future DB CHECK.
//! Hand-rolled char-by-char check — no `regex` dependency on this crate.

use serde::{Deserialize, Serialize};

use crate::SubscriptionError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanCode(String);

impl PlanCode {
    pub fn new(code: impl Into<String>) -> Result<Self, SubscriptionError> {
        let code = code.into();
        let len = code.chars().count();
        if !(3..=32).contains(&len) {
            return Err(SubscriptionError::InvalidPlanCode(code));
        }
        let body_ok = code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !body_ok {
            return Err(SubscriptionError::InvalidPlanCode(code));
        }
        Ok(Self(code))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for PlanCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical() {
        for s in [
            "free",
            "free_monthly",
            "starter_monthly",
            "pro_monthly",
            "enterprise_monthly",
            "abc",
            "a_b_c_1_2_3",
            "0123456789012345678901234567890a", // 32 chars
        ] {
            assert!(PlanCode::new(s).is_ok(), "expected `{s}` to validate");
        }
    }

    #[test]
    fn rejects_garbage() {
        for s in [
            "ab",                                 // too short
            "",                                   // empty
            "UPPER",                              // uppercase
            "with-hyphen",                        // hyphen
            "with space",                         // space
            "wíth_accent",                        // non-ascii
            "0123456789012345678901234567890abc", // 34 chars (over)
        ] {
            assert!(PlanCode::new(s).is_err(), "expected `{s}` to fail");
        }
    }
}
