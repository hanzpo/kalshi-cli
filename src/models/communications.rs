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
