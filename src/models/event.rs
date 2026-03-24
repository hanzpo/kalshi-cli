use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_ticker: Option<String>,
    pub series_ticker: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub mutually_exclusive: Option<bool>,
    pub strike_date: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    pub events: Option<Vec<Event>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventResponse {
    pub event: Event,
}

impl TableDisplay for Event {
    fn headers() -> Vec<&'static str> {
        vec!["Event Ticker", "Series", "Title", "Category", "Status"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.event_ticker),
            format_opt(&self.series_ticker),
            self.title
                .as_ref()
                .map(|t| {
                    if t.len() > 50 {
                        format!("{}...", &t[..47])
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_else(|| "-".to_string()),
            format_opt(&self.category),
            format_opt(&self.status),
        ]
    }
}

