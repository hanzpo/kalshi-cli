use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub ticker: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub frequency: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesListResponse {
    pub series: Option<Vec<Series>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesResponse {
    pub series: Series,
}

impl TableDisplay for Series {
    fn headers() -> Vec<&'static str> {
        vec!["Ticker", "Title", "Category", "Frequency"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.ticker),
            format_opt(&self.title),
            format_opt(&self.category),
            format_opt(&self.frequency),
        ]
    }
}
