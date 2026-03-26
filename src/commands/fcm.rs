use anyhow::Result;

use crate::cli::FcmCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(client: &KalshiClient, cmd: FcmCmd, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    match cmd {
        FcmCmd::Order {
            subtrader_id,
            cursor,
            event_ticker,
            ticker,
            min_ts,
            max_ts,
            status,
            limit,
        } => {
            let mut query = vec![("subtrader_id", subtrader_id.as_str())];
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            if let Some(ref e) = event_ticker {
                query.push(("event_ticker", e.as_str()));
            }
            if let Some(ref t) = ticker {
                query.push(("ticker", t.as_str()));
            }
            let min_str = min_ts.map(|t| t.to_string());
            let max_str = max_ts.map(|t| t.to_string());
            if let Some(ref t) = min_str {
                query.push(("min_ts", t.as_str()));
            }
            if let Some(ref t) = max_str {
                query.push(("max_ts", t.as_str()));
            }
            if let Some(ref s) = status {
                query.push(("status", s.as_str()));
            }
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            let resp: serde_json::Value = client.get("/fcm/orders", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
        FcmCmd::Position {
            subtrader_id,
            ticker,
            event_ticker,
            count_filter,
            settlement_status,
            limit,
            cursor,
        } => {
            let mut query = vec![("subtrader_id", subtrader_id.as_str())];
            if let Some(ref t) = ticker {
                query.push(("ticker", t.as_str()));
            }
            if let Some(ref e) = event_ticker {
                query.push(("event_ticker", e.as_str()));
            }
            if let Some(ref c) = count_filter {
                query.push(("count_filter", c.as_str()));
            }
            if let Some(ref s) = settlement_status {
                query.push(("settlement_status", s.as_str()));
            }
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: serde_json::Value = client.get("/fcm/positions", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
