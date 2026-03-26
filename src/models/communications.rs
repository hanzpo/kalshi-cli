use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

// RFQ
#[derive(Debug, Serialize)]
pub struct CreateRfqRequest {
    pub ticker: String,
    pub count: i64,
    pub side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rfq {
    pub rfq_id: Option<String>,
    pub ticker: Option<String>,
    pub side: Option<String>,
    pub count: Option<i64>,
    pub status: Option<String>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RfqResponse {
    pub rfq: Option<Rfq>,
}

#[derive(Debug, Deserialize)]
pub struct RfqsResponse {
    pub rfqs: Option<Vec<Rfq>>,
}

impl TableDisplay for Rfq {
    fn headers() -> Vec<&'static str> {
        vec!["RFQ ID", "Ticker", "Side", "Count", "Status", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.rfq_id),
            format_opt(&self.ticker),
            format_opt(&self.side),
            format_opt(&self.count),
            format_opt(&self.status),
            format_opt(&self.created_time),
        ]
    }
}

// Quote
#[derive(Debug, Serialize)]
pub struct CreateQuoteRequest {
    pub rfq_id: String,
    pub price: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub quote_id: Option<String>,
    pub rfq_id: Option<String>,
    pub price: Option<i64>,
    pub status: Option<String>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub quote: Option<Quote>,
}

#[derive(Debug, Deserialize)]
pub struct QuotesResponse {
    pub quotes: Option<Vec<Quote>>,
}

impl TableDisplay for Quote {
    fn headers() -> Vec<&'static str> {
        vec!["Quote ID", "RFQ ID", "Price", "Status", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.quote_id),
            format_opt(&self.rfq_id),
            format_opt(&self.price),
            format_opt(&self.status),
            format_opt(&self.created_time),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Rfq tests ──

    #[test]
    fn rfq_headers() {
        let h = Rfq::headers();
        assert_eq!(h, vec!["RFQ ID", "Ticker", "Side", "Count", "Status", "Created"]);
    }

    #[test]
    fn rfq_row_all_some() {
        let rfq = Rfq {
            rfq_id: Some("rfq-1".to_string()),
            ticker: Some("MKT-A".to_string()),
            side: Some("yes".to_string()),
            count: Some(100),
            status: Some("open".to_string()),
            created_time: Some("2025-06-01T12:00:00Z".to_string()),
            extra: Default::default(),
        };
        let row = rfq.row();
        assert_eq!(row.len(), 6);
        assert_eq!(row[0], "rfq-1");
        assert_eq!(row[1], "MKT-A");
        assert_eq!(row[2], "yes");
        assert_eq!(row[3], "100");
        assert_eq!(row[4], "open");
        assert_eq!(row[5], "2025-06-01T12:00:00Z");
    }

    #[test]
    fn rfq_row_all_none() {
        let rfq = Rfq {
            rfq_id: None,
            ticker: None,
            side: None,
            count: None,
            status: None,
            created_time: None,
            extra: Default::default(),
        };
        let row = rfq.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    // ── Quote tests ──

    #[test]
    fn quote_headers() {
        let h = Quote::headers();
        assert_eq!(h, vec!["Quote ID", "RFQ ID", "Price", "Status", "Created"]);
    }

    #[test]
    fn quote_row_all_some() {
        let q = Quote {
            quote_id: Some("q-1".to_string()),
            rfq_id: Some("rfq-1".to_string()),
            price: Some(55),
            status: Some("filled".to_string()),
            created_time: Some("2025-07-01T00:00:00Z".to_string()),
            extra: Default::default(),
        };
        let row = q.row();
        assert_eq!(row.len(), 5);
        assert_eq!(row[0], "q-1");
        assert_eq!(row[1], "rfq-1");
        assert_eq!(row[2], "55");
        assert_eq!(row[3], "filled");
        assert_eq!(row[4], "2025-07-01T00:00:00Z");
    }

    #[test]
    fn quote_row_all_none() {
        let q = Quote {
            quote_id: None,
            rfq_id: None,
            price: None,
            status: None,
            created_time: None,
            extra: Default::default(),
        };
        let row = q.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    // ── Serialization tests ──

    #[test]
    fn create_rfq_request_serialization() {
        let req = CreateRfqRequest {
            ticker: "MKT-XYZ".to_string(),
            count: 50,
            side: "yes".to_string(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["ticker"], "MKT-XYZ");
        assert_eq!(json["count"], 50);
        assert_eq!(json["side"], "yes");
    }

    #[test]
    fn create_quote_request_serialization() {
        let req = CreateQuoteRequest {
            rfq_id: "rfq-abc".to_string(),
            price: 42,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["rfq_id"], "rfq-abc");
        assert_eq!(json["price"], 42);
    }
}
