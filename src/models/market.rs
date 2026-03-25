use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
    pub market_type: Option<String>,
    pub result: Option<String>,

    // Prices — the API returns dollar strings like "0.6500"
    pub yes_bid_dollars: Option<String>,
    pub yes_ask_dollars: Option<String>,
    pub no_bid_dollars: Option<String>,
    pub no_ask_dollars: Option<String>,
    pub last_price_dollars: Option<String>,
    pub previous_price_dollars: Option<String>,
    pub previous_yes_bid_dollars: Option<String>,
    pub previous_yes_ask_dollars: Option<String>,
    pub notional_value_dollars: Option<String>,
    pub liquidity_dollars: Option<String>,

    // Sizes
    pub yes_bid_size_fp: Option<String>,
    pub yes_ask_size_fp: Option<String>,

    // Volume / OI — also dollar strings
    pub volume_fp: Option<String>,
    pub volume_24h_fp: Option<String>,
    pub open_interest_fp: Option<String>,

    // Timing
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub expected_expiration_time: Option<String>,
    pub expiration_time: Option<String>,
    pub created_time: Option<String>,

    // Other
    pub yes_sub_title: Option<String>,
    pub no_sub_title: Option<String>,
    pub rules_primary: Option<String>,
    pub rules_secondary: Option<String>,
    pub settlement_timer_seconds: Option<i64>,
    pub strike_type: Option<String>,
    pub custom_strike: Option<serde_json::Value>,
    pub can_close_early: Option<bool>,
    pub response_price_units: Option<String>,
    pub tick_size: Option<f64>,
    pub price_level_structure: Option<String>,
    pub fractional_trading_enabled: Option<bool>,
    pub is_provisional: Option<bool>,

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

/// Trim a dollar string like "0.6500" to "0.65", or "0.0000" to "-"
fn fmt_price(val: &Option<String>) -> String {
    match val {
        Some(s) if s == "0.0000" || s.is_empty() => "-".to_string(),
        Some(s) => {
            // Parse and re-format to trim trailing zeros
            s.parse::<f64>()
                .map(|v| format!("{:.2}", v))
                .unwrap_or_else(|_| s.clone())
        }
        None => "-".to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

impl TableDisplay for Market {
    fn headers() -> Vec<&'static str> {
        vec![
            "Ticker", "Title", "Status", "Yes Bid", "Yes Ask", "Last", "Volume", "OI",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            truncate(&format_opt(&self.ticker), 40),
            truncate(&format_opt(&self.title), 45),
            format_opt(&self.status),
            fmt_price(&self.yes_bid_dollars),
            fmt_price(&self.yes_ask_dollars),
            fmt_price(&self.last_price_dollars),
            fmt_price(&self.volume_fp),
            fmt_price(&self.open_interest_fp),
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
