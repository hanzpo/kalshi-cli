use serde::{Deserialize, Serialize};

use crate::color;
use crate::models::common::{flexible_f64, format_opt};
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: Option<i64>,
    pub portfolio_value: Option<i64>,
    pub payout: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BalanceResponse {
    pub balance: Option<i64>,
    pub portfolio_value: Option<i64>,
    pub payout: Option<i64>,
}

impl TableDisplay for Balance {
    fn headers() -> Vec<&'static str> {
        vec!["Balance", "Portfolio Value", "Payout"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.balance
                .map(|c| format!("${:.2}", c as f64 / 100.0))
                .unwrap_or("-".into()),
            self.portfolio_value
                .map(|c| format!("${:.2}", c as f64 / 100.0))
                .unwrap_or("-".into()),
            self.payout
                .map(|c| format!("${:.2}", c as f64 / 100.0))
                .unwrap_or("-".into()),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub ticker: Option<String>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub market_exposure: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub position: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub realized_pnl: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub resting_orders_count: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub total_traded: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub fees_paid: Option<f64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PositionsResponse {
    pub market_positions: Option<Vec<Position>>,
    pub cursor: Option<String>,
}

impl TableDisplay for Position {
    fn headers() -> Vec<&'static str> {
        vec![
            "Ticker",
            "Position",
            "Exposure",
            "Realized PnL",
            "Fees Paid",
            "Total Traded",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.ticker),
            self.position
                .map(|v| v.to_string())
                .or_else(|| self.extra.get("position_fp").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| "-".to_string()),
            self.market_exposure
                .map(|v| format!("${:.2}", v))
                .or_else(|| self.extra.get("market_exposure_dollars").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).map(|v| format!("${:.2}", v)))
                .unwrap_or_else(|| "-".to_string()),
            self.realized_pnl
                .map(|v| format!("${:.2}", v))
                .or_else(|| self.extra.get("realized_pnl_dollars").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).map(|v| format!("${:.2}", v)))
                .unwrap_or_else(|| "-".to_string()),
            self.fees_paid
                .map(|v| format!("${:.2}", v))
                .or_else(|| self.extra.get("fees_paid_dollars").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).map(|v| format!("${:.2}", v)))
                .unwrap_or_else(|| "-".to_string()),
            self.total_traded
                .map(|v| format!("${:.2}", v))
                .or_else(|| self.extra.get("total_traded_dollars").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).map(|v| format!("${:.2}", v)))
                .unwrap_or_else(|| "-".to_string()),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        let pnl_dollars = self.realized_pnl
            .or_else(|| self.extra.get("realized_pnl_dollars").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()));
        if let Some(pnl) = pnl_dollars {
            let cents = (pnl * 100.0).round() as i64;
            row[3] = color::color_pnl(cents, c);
        }
        row
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: Option<String>,
    pub order_id: Option<String>,
    pub ticker: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub count: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub yes_price: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub no_price: Option<f64>,
    pub is_taker: Option<bool>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub fee_cost: Option<f64>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct FillsResponse {
    pub fills: Option<Vec<Fill>>,
    pub cursor: Option<String>,
}

impl TableDisplay for Fill {
    fn headers() -> Vec<&'static str> {
        vec![
            "Fill ID",
            "Ticker",
            "Side",
            "Action",
            "Count",
            "Yes Price",
            "No Price",
            "Fee",
            "Time",
        ]
    }

    fn row(&self) -> Vec<String> {
        let count = self.count.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("count_fp")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        let yes_price = self.yes_price.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("yes_price_dollars")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        let no_price = self.no_price.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("no_price_dollars")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        vec![
            format_opt(&self.fill_id),
            format_opt(&self.ticker),
            format_opt(&self.side),
            format_opt(&self.action),
            format_opt(&count),
            format_opt(&yes_price),
            format_opt(&no_price),
            format_opt(&self.fee_cost),
            format_opt(&self.created_time),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref side) = self.side {
            row[2] = color::color_side(side, c);
        }
        if let Some(ref action) = self.action {
            row[3] = color::color_action(action, c);
        }
        row
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub ticker: Option<String>,
    pub market_result: Option<String>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub yes_count: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub no_count: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub revenue: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub fee_cost: Option<f64>,
    pub settled_time: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SettlementsResponse {
    pub settlements: Option<Vec<Settlement>>,
    pub cursor: Option<String>,
}

impl TableDisplay for Settlement {
    fn headers() -> Vec<&'static str> {
        vec![
            "Ticker",
            "Result",
            "Yes Count",
            "No Count",
            "Revenue",
            "Fee",
            "Settled",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.ticker),
            format_opt(&self.market_result),
            format_opt(&self.yes_count),
            format_opt(&self.no_count),
            format_opt(&self.revenue),
            format_opt(&self.fee_cost),
            format_opt(&self.settled_time),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref result) = self.market_result {
            row[1] = color::color_result(result, c);
        }
        if let Some(dollars) = self.revenue {
            let cents = (dollars * 100.0).round() as i64;
            row[4] = color::color_pnl(cents, c);
        }
        row
    }
}

#[derive(Debug, Deserialize)]
pub struct RestingValueResponse {
    pub total_resting_order_value: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_headers() {
        let headers = Balance::headers();
        assert_eq!(headers, vec!["Balance", "Portfolio Value", "Payout"]);
    }

    #[test]
    fn test_balance_row() {
        let balance = Balance {
            balance: Some(10000),
            portfolio_value: Some(5000),
            payout: Some(2000),
        };
        let row = balance.row();
        assert_eq!(row[0], "$100.00");
        assert_eq!(row[1], "$50.00");
        assert_eq!(row[2], "$20.00");
    }

    #[test]
    fn test_balance_row_all_none() {
        let balance = Balance {
            balance: None,
            portfolio_value: None,
            payout: None,
        };
        let row = balance.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_position_headers() {
        let headers = Position::headers();
        assert_eq!(
            headers,
            vec![
                "Ticker",
                "Position",
                "Exposure",
                "Realized PnL",
                "Fees Paid",
                "Total Traded"
            ]
        );
    }

    #[test]
    fn test_position_row() {
        let pos = Position {
            ticker: Some("MKT-1".to_string()),
            market_exposure: Some(500.0),
            position: Some(10.0),
            realized_pnl: Some(-200.0),
            resting_orders_count: Some(3.0),
            total_traded: Some(1000.0),
            fees_paid: Some(50.0),
            extra: std::collections::HashMap::new(),
        };
        let row = pos.row();
        assert_eq!(row[0], "MKT-1");
        assert_eq!(row[1], "10");
        assert_eq!(row[2], "$500.00");
        assert_eq!(row[3], "$-200.00");
        assert_eq!(row[4], "$50.00");
        assert_eq!(row[5], "$1000.00");
    }

    #[test]
    fn test_position_row_all_none() {
        let pos = Position {
            ticker: None,
            market_exposure: None,
            position: None,
            realized_pnl: None,
            resting_orders_count: None,
            total_traded: None,
            fees_paid: None,
            extra: std::collections::HashMap::new(),
        };
        let row = pos.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_fill_headers() {
        let headers = Fill::headers();
        assert_eq!(
            headers,
            vec![
                "Fill ID",
                "Ticker",
                "Side",
                "Action",
                "Count",
                "Yes Price",
                "No Price",
                "Fee",
                "Time"
            ]
        );
    }

    #[test]
    fn test_fill_row() {
        let fill = Fill {
            fill_id: Some("fill-1".to_string()),
            order_id: Some("order-1".to_string()),
            ticker: Some("MKT-1".to_string()),
            side: Some("yes".to_string()),
            action: Some("buy".to_string()),
            count: Some(5.0),
            yes_price: Some(65.0),
            no_price: Some(35.0),
            is_taker: Some(true),
            fee_cost: Some(10.0),
            created_time: Some("2026-01-01".to_string()),
            extra: std::collections::HashMap::new(),
        };
        let row = fill.row();
        assert_eq!(row[0], "fill-1");
        assert_eq!(row[1], "MKT-1");
        assert_eq!(row[2], "yes");
        assert_eq!(row[3], "buy");
        assert_eq!(row[4], "5");
        assert_eq!(row[5], "65");
        assert_eq!(row[6], "35");
        assert_eq!(row[7], "10");
        assert_eq!(row[8], "2026-01-01");
    }

    #[test]
    fn test_fill_row_all_none() {
        let fill = Fill {
            fill_id: None,
            order_id: None,
            ticker: None,
            side: None,
            action: None,
            count: None,
            yes_price: None,
            no_price: None,
            is_taker: None,
            fee_cost: None,
            created_time: None,
            extra: std::collections::HashMap::new(),
        };
        let row = fill.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_settlement_headers() {
        let headers = Settlement::headers();
        assert_eq!(
            headers,
            vec![
                "Ticker",
                "Result",
                "Yes Count",
                "No Count",
                "Revenue",
                "Fee",
                "Settled"
            ]
        );
    }

    #[test]
    fn test_settlement_row() {
        let s = Settlement {
            ticker: Some("MKT-1".to_string()),
            market_result: Some("yes".to_string()),
            yes_count: Some(10.0),
            no_count: Some(0.0),
            revenue: Some(10.0),
            fee_cost: Some(0.5),
            settled_time: Some("2026-02-01".to_string()),
            extra: std::collections::HashMap::new(),
        };
        let row = s.row();
        assert_eq!(row[0], "MKT-1");
        assert_eq!(row[1], "yes");
        assert_eq!(row[2], "10");
        assert_eq!(row[3], "0");
        assert_eq!(row[4], "10");
        assert_eq!(row[5], "0.5");
        assert_eq!(row[6], "2026-02-01");
    }

    #[test]
    fn test_settlement_row_all_none() {
        let s = Settlement {
            ticker: None,
            market_result: None,
            yes_count: None,
            no_count: None,
            revenue: None,
            fee_cost: None,
            settled_time: None,
            extra: std::collections::HashMap::new(),
        };
        let row = s.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }
}
