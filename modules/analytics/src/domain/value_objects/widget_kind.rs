use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AnalyticsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetKind {
    KpiCard,
    LineChart,
    BarChart,
    PieChart,
    Table,
}

impl fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WidgetKind::KpiCard => "kpi_card",
            WidgetKind::LineChart => "line_chart",
            WidgetKind::BarChart => "bar_chart",
            WidgetKind::PieChart => "pie_chart",
            WidgetKind::Table => "table",
        };
        f.write_str(s)
    }
}

impl FromStr for WidgetKind {
    type Err = AnalyticsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "kpi_card" => Ok(Self::KpiCard),
            "line_chart" => Ok(Self::LineChart),
            "bar_chart" => Ok(Self::BarChart),
            "pie_chart" => Ok(Self::PieChart),
            "table" => Ok(Self::Table),
            other => Err(AnalyticsError::InvalidWidgetKind(other.into())),
        }
    }
}
