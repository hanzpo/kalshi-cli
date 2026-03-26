use anyhow::{Context, Result};

use crate::cli::CollectionCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(client: &KalshiClient, cmd: CollectionCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        CollectionCmd::List {
            status,
            associated_event_ticker,
            series_ticker,
            limit,
            cursor,
            all: _,
        } => {
            let mut query = Vec::new();
            if let Some(ref s) = status {
                query.push(("status", s.as_str()));
            }
            if let Some(ref e) = associated_event_ticker {
                query.push(("associated_event_ticker", e.as_str()));
            }
            if let Some(ref s) = series_ticker {
                query.push(("series_ticker", s.as_str()));
            }
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: serde_json::Value = client
                .get("/multivariate_event_collections", &query)
                .await?;
            print_json(&resp, out.no_pager)?;
        }
        CollectionCmd::Get { ticker } => {
            let path = format!("/multivariate_event_collections/{}", ticker);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        CollectionCmd::CreateMarket {
            ticker,
            file,
            with_market_payload,
        } => {
            client.require_auth()?;
            let contents = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read {}", file.display()))?;
            let selected_markets: serde_json::Value =
                serde_json::from_str(&contents).context("Failed to parse JSON file")?;
            let body = serde_json::json!({
                "selected_markets": selected_markets,
                "with_market_payload": with_market_payload,
            });
            let path = format!("/multivariate_event_collections/{}", ticker);
            let resp: serde_json::Value = client.post(&path, &body).await?;
            print_json(&resp, out.no_pager)?;
        }
        CollectionCmd::Lookup { ticker, file } => {
            client.require_auth()?;
            let contents = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read {}", file.display()))?;
            let selected_markets: serde_json::Value =
                serde_json::from_str(&contents).context("Failed to parse JSON file")?;
            let body = serde_json::json!({
                "selected_markets": selected_markets,
            });
            let path = format!("/multivariate_event_collections/{}/lookup", ticker);
            let resp: serde_json::Value = client.put(&path, &body).await?;
            print_json(&resp, out.no_pager)?;
        }
        CollectionCmd::LookupHistory {
            ticker,
            lookback_seconds,
        } => {
            let path = format!("/multivariate_event_collections/{}/lookup", ticker);
            let mut query = Vec::new();
            let lookback_str = lookback_seconds.map(|l| l.to_string());
            if let Some(ref l) = lookback_str {
                query.push(("lookback_seconds", l.as_str()));
            }
            let resp: serde_json::Value = client.get(&path, &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
