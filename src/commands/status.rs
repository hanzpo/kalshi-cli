use anyhow::Result;

use crate::client::KalshiClient;
use crate::models::exchange::ExchangeStatusResponse;
use crate::models::order::OrdersResponse;
use crate::models::portfolio::{BalanceResponse, FillsResponse, PositionsResponse};
use crate::output::{OutputConfig, print_table};

pub async fn execute(client: &KalshiClient, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    // Use small limits — this is a summary view, we only need counts.
    // Positions/orders max per page: 1000/200 respectively.
    let (exchange, balance, positions, orders, fills) = tokio::try_join!(
        client.get::<ExchangeStatusResponse>("/exchange/status", &[]),
        client.get::<BalanceResponse>("/portfolio/balance", &[]),
        client.get::<PositionsResponse>("/portfolio/positions", &[("limit", "200")]),
        client.get::<OrdersResponse>(
            "/portfolio/orders",
            &[("status", "resting"), ("limit", "200")]
        ),
        client.get::<FillsResponse>("/portfolio/fills", &[("limit", "5")]),
    )?;

    let exchange_active = exchange
        .exchange_active
        .map_or("Unknown", |v| if v { "Active" } else { "Inactive" });
    let trading_active = exchange
        .trading_active
        .map_or("Unknown", |v| if v { "Active" } else { "Inactive" });

    let balance_cents = balance.balance.unwrap_or(0);
    let portfolio_cents = balance.portfolio_value.unwrap_or(0);
    let position_count = positions.market_positions.as_ref().map_or(0, |p| p.len());
    let order_count = orders.orders.as_ref().map_or(0, |o| o.len());

    eprintln!(
        "Exchange: {} | Trading: {}",
        exchange_active, trading_active
    );
    eprintln!(
        "Balance: ${:.2} | Portfolio: ${:.2}",
        balance_cents as f64 / 100.0,
        portfolio_cents as f64 / 100.0
    );
    eprintln!(
        "Open Positions: {} | Resting Orders: {}",
        position_count, order_count
    );

    let fills_list = fills.fills.unwrap_or_default();
    if !fills_list.is_empty() {
        eprintln!("Recent Fills:");
        print_table(&fills_list, out.no_pager, out.color)?;
    } else {
        eprintln!("Recent Fills: None");
    }

    Ok(())
}
