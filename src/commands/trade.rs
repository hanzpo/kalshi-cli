use anyhow::{Result, bail};

use crate::client::KalshiClient;
use crate::confirm;
use crate::models::order::{
    BatchCancelRequest, BatchCancelResponse, CreateOrderRequest, OrderResponse,
};
use crate::models::portfolio::PositionsResponse;
use crate::output::{OutputConfig, output_one};

pub async fn execute_buy(
    client: &KalshiClient,
    ticker: &str,
    quantity: i64,
    _yes: bool,
    no: bool,
    price: Option<i64>,
    out: &OutputConfig,
) -> Result<()> {
    client.require_auth()?;

    let side = if no { "no" } else { "yes" };

    let price_display = price.map_or("market".to_string(), |p| format!("{}c", p));
    let msg = format!(
        "Buy {} {} @ {} on {}?",
        quantity, side, price_display, ticker
    );
    if !confirm::confirm(&msg, false)? {
        eprintln!("Cancelled.");
        return Ok(());
    }

    let req = CreateOrderRequest {
        ticker: ticker.to_string(),
        side: side.to_string(),
        action: "buy".to_string(),
        count: Some(quantity),
        yes_price: if side == "yes" { price } else { None },
        no_price: if side == "no" { price } else { None },
        time_in_force: None,
        expiration_ts: None,
        client_order_id: None,
        post_only: None,
        reduce_only: None,
        buy_max_cost: None,
        order_group_id: None,
    };

    let resp: OrderResponse = client.post("/portfolio/orders", &req).await?;
    output_one(&resp.order, out)?;
    Ok(())
}

pub async fn execute_sell(
    client: &KalshiClient,
    ticker: &str,
    quantity: i64,
    _yes: bool,
    no: bool,
    price: Option<i64>,
    out: &OutputConfig,
) -> Result<()> {
    client.require_auth()?;

    let side = if no { "no" } else { "yes" };

    let price_display = price.map_or("market".to_string(), |p| format!("{}c", p));
    let msg = format!(
        "Sell {} {} @ {} on {}?",
        quantity, side, price_display, ticker
    );
    if !confirm::confirm(&msg, false)? {
        eprintln!("Cancelled.");
        return Ok(());
    }

    let req = CreateOrderRequest {
        ticker: ticker.to_string(),
        side: side.to_string(),
        action: "sell".to_string(),
        count: Some(quantity),
        yes_price: if side == "yes" { price } else { None },
        no_price: if side == "no" { price } else { None },
        time_in_force: None,
        expiration_ts: None,
        client_order_id: None,
        post_only: None,
        reduce_only: None,
        buy_max_cost: None,
        order_group_id: None,
    };

    let resp: OrderResponse = client.post("/portfolio/orders", &req).await?;
    output_one(&resp.order, out)?;
    Ok(())
}

pub async fn execute_close(client: &KalshiClient, ticker: &str, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    let positions: PositionsResponse = client
        .get(
            "/portfolio/positions",
            &[("ticker", ticker), ("limit", "10")],
        )
        .await?;

    let pos_list = positions.market_positions.unwrap_or_default();
    let pos = pos_list
        .iter()
        .find(|p| p.ticker.as_deref() == Some(ticker));

    let pos = match pos {
        Some(p) => p,
        None => bail!("No open position found for {}", ticker),
    };

    let count = pos.position.unwrap_or(0);
    if count == 0 {
        eprintln!("Position is already flat on {}", ticker);
        return Ok(());
    }

    let msg = format!("Close position of {} on {}?", count, ticker);
    if !confirm::confirm(&msg, false)? {
        eprintln!("Cancelled.");
        return Ok(());
    }

    // To close: sell if positive position, buy if negative
    let (action, side, qty) = if count > 0 {
        ("sell", "yes", count)
    } else {
        ("buy", "yes", -count)
    };

    let req = CreateOrderRequest {
        ticker: ticker.to_string(),
        side: side.to_string(),
        action: action.to_string(),
        count: Some(qty),
        yes_price: None,
        no_price: None,
        time_in_force: None,
        expiration_ts: None,
        client_order_id: None,
        post_only: None,
        reduce_only: Some(true),
        buy_max_cost: None,
        order_group_id: None,
    };

    let resp: OrderResponse = client.post("/portfolio/orders", &req).await?;
    output_one(&resp.order, out)?;
    Ok(())
}

pub async fn execute_cancel_all(
    client: &KalshiClient,
    ticker_filter: Option<&str>,
    _out: &OutputConfig,
) -> Result<()> {
    client.require_auth()?;

    let msg = match ticker_filter {
        Some(t) => format!("Cancel all resting orders on {}?", t),
        None => "Cancel all resting orders?".to_string(),
    };

    if !confirm::confirm(&msg, false)? {
        eprintln!("Cancelled.");
        return Ok(());
    }

    let req = BatchCancelRequest {
        ticker: ticker_filter.map(|s| s.to_string()),
        order_ids: None,
    };

    let resp: BatchCancelResponse = client
        .delete_with_body("/portfolio/orders/batched", &req)
        .await?;

    let count = resp.orders_canceled.unwrap_or(0);
    eprintln!("Cancelled {} orders.", count);
    Ok(())
}
