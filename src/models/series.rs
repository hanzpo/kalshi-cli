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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_returns_four_columns() {
        let h = Series::headers();
        assert_eq!(h, vec!["Ticker", "Title", "Category", "Frequency"]);
    }

    #[test]
    fn row_all_some() {
        let s = Series {
            ticker: Some("SER-1".to_string()),
            title: Some("Weather Series".to_string()),
            category: Some("climate".to_string()),
            frequency: Some("daily".to_string()),
            tags: Some(vec!["weather".to_string()]),
            extra: Default::default(),
        };
        let row = s.row();
        assert_eq!(row, vec!["SER-1", "Weather Series", "climate", "daily"]);
    }

    #[test]
    fn row_all_none() {
        let s = Series {
            ticker: None,
            title: None,
            category: None,
            frequency: None,
            tags: None,
            extra: Default::default(),
        };
        let row = s.row();
        assert_eq!(row, vec!["-", "-", "-", "-"]);
    }

    #[test]
    fn row_partial_fields() {
        let s = Series {
            ticker: Some("SER-2".to_string()),
            title: None,
            category: Some("politics".to_string()),
            frequency: None,
            tags: None,
            extra: Default::default(),
        };
        let row = s.row();
        assert_eq!(row[0], "SER-2");
        assert_eq!(row[1], "-");
        assert_eq!(row[2], "politics");
        assert_eq!(row[3], "-");
    }
}
