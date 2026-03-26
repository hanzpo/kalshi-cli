use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMarket {
    pub ticker: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub close_time: Option<String>,
    pub result: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalMarketsResponse {
    pub markets: Option<Vec<HistoricalMarket>>,
    pub cursor: Option<String>,
}

impl TableDisplay for HistoricalMarket {
    fn headers() -> Vec<&'static str> {
        vec!["Ticker", "Title", "Status", "Close Time", "Result"]
    }

    fn row(&self) -> Vec<String> {
        let title = self
            .title
            .as_ref()
            .map(|t| {
                if t.chars().count() > 50 {
                    format!("{}...", &t.chars().take(47).collect::<String>())
                } else {
                    t.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string());
        vec![
            format_opt(&self.ticker),
            title,
            format_opt(&self.status),
            format_opt(&self.close_time),
            format_opt(&self.result),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct CutoffResponse {
    pub cutoff_ts: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_returns_five_columns() {
        let h = HistoricalMarket::headers();
        assert_eq!(h.len(), 5);
        assert_eq!(h, vec!["Ticker", "Title", "Status", "Close Time", "Result"]);
    }

    #[test]
    fn row_all_some() {
        let m = HistoricalMarket {
            ticker: Some("HIST-1".to_string()),
            title: Some("Will X happen?".to_string()),
            status: Some("settled".to_string()),
            close_time: Some("2025-03-01T00:00:00Z".to_string()),
            result: Some("yes".to_string()),
            extra: Default::default(),
        };
        let row = m.row();
        assert_eq!(row[0], "HIST-1");
        assert_eq!(row[1], "Will X happen?");
        assert_eq!(row[2], "settled");
        assert_eq!(row[3], "2025-03-01T00:00:00Z");
        assert_eq!(row[4], "yes");
    }

    #[test]
    fn row_all_none() {
        let m = HistoricalMarket {
            ticker: None,
            title: None,
            status: None,
            close_time: None,
            result: None,
            extra: Default::default(),
        };
        let row = m.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    #[test]
    fn row_partial_fields() {
        let m = HistoricalMarket {
            ticker: Some("HIST-2".to_string()),
            title: None,
            status: Some("finalized".to_string()),
            close_time: None,
            result: Some("no".to_string()),
            extra: Default::default(),
        };
        let row = m.row();
        assert_eq!(row[0], "HIST-2");
        assert_eq!(row[1], "-");
        assert_eq!(row[2], "finalized");
        assert_eq!(row[3], "-");
        assert_eq!(row[4], "no");
    }
}
