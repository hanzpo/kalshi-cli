use serde::{Deserialize, Serialize};

use crate::color;
use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
            "Ticker",
            "Title",
            "Status",
            "Yes Bid",
            "Yes Ask",
            "Last Price",
            "Volume",
            "Open Int",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.ticker),
            self.title
                .as_ref()
                .map(|t| {
                    if t.chars().count() > 50 {
                        let truncated: String = t.chars().take(47).collect();
                        format!("{truncated}...")
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

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref status) = self.status {
            row[2] = color::color_status(status, c);
        }
        row
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
        vec![
            "Trade ID",
            "Ticker",
            "Count",
            "Yes Price",
            "No Price",
            "Taker Side",
            "Time",
        ]
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

// ── Search API (internal/undocumented v1 endpoint) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMarket {
    pub ticker: Option<String>,
    pub yes_subtitle: Option<String>,
    pub yes_bid: Option<i64>,
    pub yes_ask: Option<i64>,
    pub last_price: Option<i64>,
    pub volume: Option<i64>,
    pub close_ts: Option<String>,
    pub open_ts: Option<String>,
    pub result: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    pub series_ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub event_title: Option<String>,
    pub category: Option<String>,
    pub total_volume: Option<i64>,
    pub active_market_count: Option<i64>,
    pub markets: Option<Vec<SearchMarket>>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl TableDisplay for SearchEvent {
    fn headers() -> Vec<&'static str> {
        vec![
            "Event",
            "Title",
            "Category",
            "Markets",
            "Volume",
            "Top Market",
            "Yes Bid",
            "Yes Ask",
        ]
    }

    fn row(&self) -> Vec<String> {
        let title = self
            .event_title
            .as_ref()
            .map(|t| {
                if t.chars().count() > 45 {
                    let truncated: String = t.chars().take(42).collect();
                    format!("{truncated}...")
                } else {
                    t.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string());

        let first_market = self.markets.as_ref().and_then(|m| m.first());
        let top_subtitle = first_market
            .and_then(|m| m.yes_subtitle.as_deref())
            .unwrap_or("-");
        let bid = first_market
            .and_then(|m| m.yes_bid)
            .map_or("-".to_string(), |v| format!("{}¢", v));
        let ask = first_market
            .and_then(|m| m.yes_ask)
            .map_or("-".to_string(), |v| format!("{}¢", v));

        vec![
            format_opt(&self.event_ticker),
            title,
            format_opt(&self.category),
            self.active_market_count
                .map_or("-".to_string(), |v| v.to_string()),
            self.total_volume
                .map_or("-".to_string(), |v| v.to_string()),
            top_subtitle.to_string(),
            bid,
            ask,
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub total_results_count: Option<i64>,
    pub current_page: Option<Vec<SearchEvent>>,
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_market(title: Option<&str>) -> Market {
        Market {
            ticker: Some("TICKER-1".to_string()),
            event_ticker: Some("EVT-1".to_string()),
            title: title.map(|s| s.to_string()),
            status: Some("open".to_string()),
            yes_bid: Some(0.65),
            yes_ask: Some(0.70),
            no_bid: None,
            no_ask: None,
            last_price: Some(0.67),
            volume: Some(1000),
            volume_24h: None,
            open_interest: Some(500),
            result: None,
            subtitle: None,
            open_time: None,
            close_time: None,
            yes_sub_title: None,
            no_sub_title: None,
            market_type: None,
            response_price_units: None,
            notional_value: None,
            tick_size: None,
            rules_primary: None,
            rules_secondary: None,
            settlement_timer_seconds: None,
            cap_strike: None,
            floor_strike: None,
            expected_expiration_time: None,
            expiration_time: None,
            settlement_value: None,
            category: None,
            risk_limit_cents: None,
            strike_type: None,
            custom_strike: None,
            functional_strike: None,
            can_close_early: None,
            extra: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_market_headers() {
        let headers = Market::headers();
        assert_eq!(
            headers,
            vec![
                "Ticker",
                "Title",
                "Status",
                "Yes Bid",
                "Yes Ask",
                "Last Price",
                "Volume",
                "Open Int"
            ]
        );
    }

    #[test]
    fn test_market_row_basic() {
        let market = make_market(Some("Will it rain tomorrow?"));
        let row = market.row();
        assert_eq!(row[0], "TICKER-1");
        assert_eq!(row[1], "Will it rain tomorrow?");
        assert_eq!(row[2], "open");
        assert_eq!(row[3], "0.65");
        assert_eq!(row[4], "0.70");
        assert_eq!(row[5], "0.67");
        assert_eq!(row[6], "1000");
        assert_eq!(row[7], "500");
    }

    #[test]
    fn test_market_row_none_fields() {
        let mut market = make_market(None);
        market.ticker = None;
        market.yes_bid = None;
        market.yes_ask = None;
        market.last_price = None;
        market.volume = None;
        market.open_interest = None;
        market.status = None;
        let row = market.row();
        assert_eq!(row[0], "-"); // ticker
        assert_eq!(row[1], "-"); // title
        assert_eq!(row[2], "-"); // status
        assert_eq!(row[3], "-"); // yes_bid
        assert_eq!(row[4], "-"); // yes_ask
        assert_eq!(row[5], "-"); // last_price
        assert_eq!(row[6], "-"); // volume
        assert_eq!(row[7], "-"); // open_interest
    }

    #[test]
    fn test_market_title_truncation() {
        let long_title = "A".repeat(60);
        let market = make_market(Some(&long_title));
        let row = market.row();
        assert!(row[1].ends_with("..."));
        assert_eq!(row[1].chars().count(), 50); // 47 chars + "..."
    }

    #[test]
    fn test_market_title_exactly_50_chars() {
        let title = "A".repeat(50);
        let market = make_market(Some(&title));
        let row = market.row();
        assert_eq!(row[1], title);
        assert!(!row[1].ends_with("..."));
    }

    #[test]
    fn test_market_title_49_chars_no_truncation() {
        let title = "A".repeat(49);
        let market = make_market(Some(&title));
        let row = market.row();
        assert_eq!(row[1], title);
    }

    #[test]
    fn test_trade_headers() {
        let headers = Trade::headers();
        assert_eq!(
            headers,
            vec![
                "Trade ID",
                "Ticker",
                "Count",
                "Yes Price",
                "No Price",
                "Taker Side",
                "Time"
            ]
        );
    }

    #[test]
    fn test_trade_row() {
        let trade = Trade {
            ticker: Some("T1".to_string()),
            trade_id: Some("trade-123".to_string()),
            count: Some(10),
            yes_price: Some(0.55),
            no_price: Some(0.45),
            taker_side: Some("yes".to_string()),
            created_time: Some("2026-01-01".to_string()),
        };
        let row = trade.row();
        assert_eq!(row[0], "trade-123");
        assert_eq!(row[1], "T1");
        assert_eq!(row[2], "10");
        assert_eq!(row[3], "0.55");
        assert_eq!(row[4], "0.45");
        assert_eq!(row[5], "yes");
        assert_eq!(row[6], "2026-01-01");
    }

    #[test]
    fn test_trade_row_none_fields() {
        let trade = Trade {
            ticker: None,
            trade_id: None,
            count: None,
            yes_price: None,
            no_price: None,
            taker_side: None,
            created_time: None,
        };
        let row = trade.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_candlestick_headers() {
        let headers = Candlestick::headers();
        assert_eq!(
            headers,
            vec!["Start", "Open", "High", "Low", "Close", "Volume"]
        );
    }

    #[test]
    fn test_candlestick_row() {
        let candle = Candlestick {
            ticker: Some("C1".to_string()),
            period: Some("1h".to_string()),
            open: Some(0.50),
            high: Some(0.80),
            low: Some(0.40),
            close: Some(0.75),
            volume: Some(200),
            open_interest: None,
            start_period_ts: Some(1700000000),
            end_period_ts: None,
            yes_price: None,
            yes_bid: None,
            yes_ask: None,
        };
        let row = candle.row();
        assert_eq!(row[0], "1700000000");
        assert_eq!(row[1], "0.50");
        assert_eq!(row[2], "0.80");
        assert_eq!(row[3], "0.40");
        assert_eq!(row[4], "0.75");
        assert_eq!(row[5], "200");
    }

    #[test]
    fn test_candlestick_row_none_fields() {
        let candle = Candlestick {
            ticker: None,
            period: None,
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
            open_interest: None,
            start_period_ts: None,
            end_period_ts: None,
            yes_price: None,
            yes_bid: None,
            yes_ask: None,
        };
        let row = candle.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }
}
