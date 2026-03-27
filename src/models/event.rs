use serde::{Deserialize, Serialize};

use crate::color;
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
                    if t.chars().count() > 50 {
                        format!("{}...", t.chars().take(47).collect::<String>())
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_else(|| "-".to_string()),
            format_opt(&self.category),
            format_opt(&self.status),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref status) = self.status {
            row[4] = color::color_status(status, c);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_event() -> Event {
        Event {
            event_ticker: Some("EVT-1".to_string()),
            series_ticker: Some("SER-1".to_string()),
            title: Some("Will it rain?".to_string()),
            category: Some("weather".to_string()),
            status: Some("open".to_string()),
            mutually_exclusive: Some(true),
            strike_date: Some("2025-06-15".to_string()),
            extra: Default::default(),
        }
    }

    #[test]
    fn headers_returns_five_columns() {
        let h = Event::headers();
        assert_eq!(
            h,
            vec!["Event Ticker", "Series", "Title", "Category", "Status"]
        );
    }

    #[test]
    fn row_all_some() {
        let e = full_event();
        let row = e.row();
        assert_eq!(row[0], "EVT-1");
        assert_eq!(row[1], "SER-1");
        assert_eq!(row[2], "Will it rain?");
        assert_eq!(row[3], "weather");
        assert_eq!(row[4], "open");
    }

    #[test]
    fn row_all_none() {
        let e = Event {
            event_ticker: None,
            series_ticker: None,
            title: None,
            category: None,
            status: None,
            mutually_exclusive: None,
            strike_date: None,
            extra: Default::default(),
        };
        let row = e.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    #[test]
    fn row_truncates_long_title() {
        let long_title = "A".repeat(60); // 60 chars > 50
        let e = Event {
            event_ticker: Some("EVT-2".to_string()),
            series_ticker: None,
            title: Some(long_title),
            category: None,
            status: None,
            mutually_exclusive: None,
            strike_date: None,
            extra: Default::default(),
        };
        let row = e.row();
        assert_eq!(row[2].len(), 50); // 47 chars + "..."
        assert!(row[2].ends_with("..."));
    }

    #[test]
    fn row_does_not_truncate_short_title() {
        let short_title = "Short title".to_string();
        let e = Event {
            event_ticker: None,
            series_ticker: None,
            title: Some(short_title.clone()),
            category: None,
            status: None,
            mutually_exclusive: None,
            strike_date: None,
            extra: Default::default(),
        };
        let row = e.row();
        assert_eq!(row[2], short_title);
    }

    #[test]
    fn row_title_exactly_50_chars_no_truncation() {
        let title = "A".repeat(50);
        let e = Event {
            event_ticker: None,
            series_ticker: None,
            title: Some(title.clone()),
            category: None,
            status: None,
            mutually_exclusive: None,
            strike_date: None,
            extra: Default::default(),
        };
        let row = e.row();
        assert_eq!(row[2], title);
    }
}
