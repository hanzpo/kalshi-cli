use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub yes_bid: Option<f64>,
    pub yes_ask: Option<f64>,
    pub no_bid: Option<f64>,
    pub no_ask: Option<f64>,
    pub last_price: Option<f64>,
    pub volume: Option<i64>,
    pub volume_24h: Option<i64>,
    pub open_interest: Option<i64>,
    pub result: Option<String>,
    pub subtitle: Option<String>,
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub yes_sub_title: Option<String>,
    pub no_sub_title: Option<String>,
    pub market_type: Option<String>,
    pub response_price_units: Option<String>,
    pub notional_value: Option<f64>,
    pub tick_size: Option<f64>,
    pub rules_primary: Option<String>,
    pub rules_secondary: Option<String>,
    pub settlement_timer_seconds: Option<i64>,
    pub cap_strike: Option<f64>,
    pub floor_strike: Option<f64>,
    pub expected_expiration_time: Option<String>,
    pub expiration_time: Option<String>,
    pub settlement_value: Option<String>,
    pub category: Option<String>,
    pub risk_limit_cents: Option<i64>,
    pub strike_type: Option<String>,
    pub custom_strike: Option<serde_json::Value>,
    pub functional_strike: Option<String>,
    pub can_close_early: Option<bool>,
    // catch-all for any fields we haven't explicitly modeled
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MarketsResponse {
    pub markets: Option<Vec<Market>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MarketResponse {
    pub market: Market,
}

impl TableDisplay for Market {
    fn headers() -> Vec<&'static str> {
        vec![
            "Ticker", "Title", "Status", "Yes Bid", "Yes Ask", "Last Price", "Volume", "Open Int",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.ticker),
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
            format_opt(&self.status),
            self.yes_bid.map_or("-".into(), |v| format!("{:.2}", v)),
            self.yes_ask.map_or("-".into(), |v| format!("{:.2}", v)),
            self.last_price.map_or("-".into(), |v| format!("{:.2}", v)),
            format_opt(&self.volume),
            format_opt(&self.open_interest),
        ]
    }
}

// Orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub yes: Option<Vec<Vec<serde_json::Value>>>,
    pub no: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
pub struct OrderbookResponse {
    pub orderbook: Option<Orderbook>,
}

// Trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub ticker: Option<String>,
    pub trade_id: Option<String>,
    pub count: Option<i64>,
    pub yes_price: Option<f64>,
    pub no_price: Option<f64>,
    pub taker_side: Option<String>,
    pub created_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TradesResponse {
    pub trades: Option<Vec<Trade>>,
    pub cursor: Option<String>,
}

impl TableDisplay for Trade {
    fn headers() -> Vec<&'static str> {
        vec!["Trade ID", "Ticker", "Count", "Yes Price", "No Price", "Taker Side", "Time"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.trade_id),
            format_opt(&self.ticker),
            format_opt(&self.count),
            self.yes_price.map_or("-".into(), |v| format!("{:.2}", v)),
            self.no_price.map_or("-".into(), |v| format!("{:.2}", v)),
            format_opt(&self.taker_side),
            format_opt(&self.created_time),
        ]
    }
}

// Candlestick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub ticker: Option<String>,
    pub period: Option<String>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
    pub start_period_ts: Option<i64>,
    pub end_period_ts: Option<i64>,
    pub yes_price: Option<f64>,
    pub yes_bid: Option<f64>,
    pub yes_ask: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CandlesticksResponse {
    pub candlesticks: Option<Vec<Candlestick>>,
}

impl TableDisplay for Candlestick {
    fn headers() -> Vec<&'static str> {
        vec!["Start", "Open", "High", "Low", "Close", "Volume"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.start_period_ts),
            self.open.map_or("-".into(), |v| format!("{:.2}", v)),
            self.high.map_or("-".into(), |v| format!("{:.2}", v)),
            self.low.map_or("-".into(), |v| format!("{:.2}", v)),
            self.close.map_or("-".into(), |v| format!("{:.2}", v)),
            format_opt(&self.volume),
        ]
    }
}
