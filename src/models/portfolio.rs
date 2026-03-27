use serde::{Deserialize, Serialize};

use crate::color;
use crate::models::common::format_opt;
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
        vec![
            "Balance (cents)",
            "Portfolio Value (cents)",
            "Payout (cents)",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.balance),
            format_opt(&self.portfolio_value),
            format_opt(&self.payout),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub ticker: Option<String>,
    pub market_exposure: Option<i64>,
    pub position: Option<i64>,
    pub realized_pnl: Option<i64>,
    pub resting_orders_count: Option<i64>,
    pub total_traded: Option<i64>,
    pub fees_paid: Option<i64>,
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
            format_opt(&self.position),
            format_opt(&self.market_exposure),
            format_opt(&self.realized_pnl),
            format_opt(&self.fees_paid),
            format_opt(&self.total_traded),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(pnl) = self.realized_pnl {
            row[3] = color::color_pnl(pnl, c);
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
    pub count: Option<i64>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub is_taker: Option<bool>,
    pub fee_cost: Option<i64>,
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
        vec![
            format_opt(&self.fill_id),
            format_opt(&self.ticker),
            format_opt(&self.side),
            format_opt(&self.action),
            format_opt(&self.count),
            format_opt(&self.yes_price),
            format_opt(&self.no_price),
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
    pub yes_count: Option<i64>,
    pub no_count: Option<i64>,
    pub revenue: Option<i64>,
    pub fee_cost: Option<i64>,
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
        if let Some(revenue) = self.revenue {
            row[4] = color::color_pnl(revenue, c);
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
        assert_eq!(
            headers,
            vec![
                "Balance (cents)",
                "Portfolio Value (cents)",
                "Payout (cents)"
            ]
        );
    }

    #[test]
    fn test_balance_row() {
        let balance = Balance {
            balance: Some(10000),
            portfolio_value: Some(5000),
            payout: Some(2000),
        };
        let row = balance.row();
        assert_eq!(row[0], "10000");
        assert_eq!(row[1], "5000");
        assert_eq!(row[2], "2000");
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
            market_exposure: Some(500),
            position: Some(10),
            realized_pnl: Some(-200),
            resting_orders_count: Some(3),
            total_traded: Some(1000),
            fees_paid: Some(50),
            extra: std::collections::HashMap::new(),
        };
        let row = pos.row();
        assert_eq!(row[0], "MKT-1");
        assert_eq!(row[1], "10");
        assert_eq!(row[2], "500");
        assert_eq!(row[3], "-200");
        assert_eq!(row[4], "50");
        assert_eq!(row[5], "1000");
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
            count: Some(5),
            yes_price: Some(65),
            no_price: Some(35),
            is_taker: Some(true),
            fee_cost: Some(10),
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
            yes_count: Some(10),
            no_count: Some(0),
            revenue: Some(1000),
            fee_cost: Some(50),
            settled_time: Some("2026-02-01".to_string()),
            extra: std::collections::HashMap::new(),
        };
        let row = s.row();
        assert_eq!(row[0], "MKT-1");
        assert_eq!(row[1], "yes");
        assert_eq!(row[2], "10");
        assert_eq!(row[3], "0");
        assert_eq!(row[4], "1000");
        assert_eq!(row[5], "50");
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
