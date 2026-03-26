use anyhow::Result;

use crate::cli::EventCmd;
use crate::client::KalshiClient;
use crate::models::event::{EventResponse, EventsResponse};
use crate::models::market::CandlesticksResponse;
use crate::output::{OutputConfig, output, output_one, print_json};
use crate::pagination::paginated_list;

pub async fn execute(client: &KalshiClient, cmd: EventCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        EventCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            category,
            with_nested_markets,
        } => {
            paginated_list(all, limit, cursor, None, out, |page_limit, page_cursor| {
                let status = status.clone();
                let series_ticker = series_ticker.clone();
                let category = category.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref s) = status {
                        query.push(("status", s.clone()));
                    }
                    if let Some(ref s) = series_ticker {
                        query.push(("series_ticker", s.clone()));
                    }
                    if let Some(ref c) = category {
                        query.push(("category", c.clone()));
                    }
                    if with_nested_markets {
                        query.push(("with_nested_markets", "true".to_string()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: EventsResponse = client.get("/events", &query_refs).await?;
                    Ok((resp.events.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
        }
        EventCmd::Get {
            event_ticker,
            with_nested_markets,
        } => {
            let path = format!("/events/{}", event_ticker);
            let mut query = Vec::new();
            if with_nested_markets {
                query.push(("with_nested_markets", "true"));
            }
            let resp: EventResponse = client.get(&path, &query).await?;
            output_one(&resp.event, out)?;
        }
        EventCmd::Metadata { event_ticker } => {
            let path = format!("/events/{}/metadata", event_ticker);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        EventCmd::Multivariate {
            limit,
            cursor,
            series_ticker,
            collection_ticker,
            with_nested_markets,
        } => {
            let mut query = Vec::new();
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            if let Some(ref s) = series_ticker {
                query.push(("series_ticker", s.as_str()));
            }
            if let Some(ref c) = collection_ticker {
                query.push(("collection_ticker", c.as_str()));
            }
            if with_nested_markets {
                query.push(("with_nested_markets", "true"));
            }
            let resp: serde_json::Value = client.get("/events/multivariate", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
        EventCmd::Candlestick {
            event_ticker,
            series_ticker,
            start_ts,
            end_ts,
            period,
        } => {
            let path = format!(
                "/series/{}/events/{}/candlesticks",
                series_ticker, event_ticker
            );
            let mut query = Vec::new();
            let period_str = period.map(|p| p.to_string());
            let start_str = start_ts.map(|t| t.to_string());
            let end_str = end_ts.map(|t| t.to_string());
            if let Some(ref p) = period_str {
                query.push(("period_interval", p.as_str()));
            }
            if let Some(ref s) = start_str {
                query.push(("start_ts", s.as_str()));
            }
            if let Some(ref e) = end_str {
                query.push(("end_ts", e.as_str()));
            }
            let resp: CandlesticksResponse = client.get(&path, &query).await?;
            output(&resp.candlesticks.unwrap_or_default(), out)?;
        }
        EventCmd::Forecast {
            event_ticker,
            series_ticker,
            percentiles,
            start_ts,
            end_ts,
            period,
        } => {
            let path = format!(
                "/series/{}/events/{}/forecast_percentile_history",
                series_ticker, event_ticker
            );
            let period_str = period.to_string();
            let start_str = start_ts.to_string();
            let end_str = end_ts.to_string();
            let percentile_strs: Vec<String> = percentiles.iter().map(|p| p.to_string()).collect();
            let mut query: Vec<(&str, &str)> = percentile_strs
                .iter()
                .map(|p| ("percentiles", p.as_str()))
                .collect();
            query.push(("period_interval", period_str.as_str()));
            query.push(("start_ts", start_str.as_str()));
            query.push(("end_ts", end_str.as_str()));
            let resp: serde_json::Value = client.get(&path, &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
