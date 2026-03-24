use serde::{Deserialize, Serialize};

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
        vec!["Balance (cents)", "Portfolio Value (cents)", "Payout (cents)"]
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
        vec!["Ticker", "Position", "Exposure", "Realized PnL", "Fees Paid", "Total Traded"]
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
        vec!["Fill ID", "Ticker", "Side", "Action", "Count", "Yes Price", "No Price", "Fee", "Time"]
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
        vec!["Ticker", "Result", "Yes Count", "No Count", "Revenue", "Fee", "Settled"]
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
}

#[derive(Debug, Deserialize)]
pub struct RestingValueResponse {
    pub total_resting_order_value: Option<i64>,
}
