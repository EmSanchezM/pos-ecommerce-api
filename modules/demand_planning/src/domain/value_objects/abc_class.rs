//! AbcClass — Pareto-style classification of SKUs by revenue contribution.
//! A: top ~80% of revenue, B: next ~15%, C: bottom ~5%.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::DemandPlanningError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbcClass {
    A,
    B,
    C,
}

impl fmt::Display for AbcClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AbcClass::A => "A",
            AbcClass::B => "B",
            AbcClass::C => "C",
        };
        f.write_str(s)
    }
}

impl FromStr for AbcClass {
    type Err = DemandPlanningError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            other => Err(DemandPlanningError::InvalidAbcClass(other.into())),
        }
    }
}
