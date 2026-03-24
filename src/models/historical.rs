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
        vec![
            format_opt(&self.ticker),
            format_opt(&self.title),
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
