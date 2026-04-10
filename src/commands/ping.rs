use std::time::Instant;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct ExchangeStatus {
    exchange_active: Option<bool>,
    trading_active: Option<bool>,
}

pub async fn execute(demo: bool) -> Result<()> {
    let base = if demo {
        "https://demo-api.kalshi.co/trade-api/v2"
    } else {
        "https://api.elections.kalshi.com/trade-api/v2"
    };

    let client = reqwest::Client::new();
    let start = Instant::now();
    let resp = client.get(format!("{base}/exchange/status")).send().await?;
    let latency = start.elapsed();

    if !resp.status().is_success() {
        eprintln!(
            "Ping failed: HTTP {} ({}ms)",
            resp.status(),
            latency.as_millis()
        );
        return Ok(());
    }

    let status: ExchangeStatus = resp.json().await?;
    let exchange = status
        .exchange_active
        .map_or("Unknown", |v| if v { "Active" } else { "Inactive" });
    let trading = status
        .trading_active
        .map_or("Unknown", |v| if v { "Active" } else { "Inactive" });

    eprintln!(
        "Exchange: {} | Trading: {} | Latency: {}ms",
        exchange,
        trading,
        latency.as_millis()
    );

    Ok(())
}
