use anyhow::Result;

use crate::cli::EventCmd;
use crate::client::KalshiClient;
use crate::models::event::{EventResponse, EventsResponse};
use crate::output::{OutputFormat, output, output_one, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

pub async fn execute(client: &KalshiClient, cmd: EventCmd, format: &OutputFormat) -> Result<()> {
    match cmd {
        EventCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            with_nested_markets,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let events = auto_paginate(&opts, 100, |page_limit, page_cursor| {
                let status = status.clone();
                let series_ticker = series_ticker.clone();
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
            output(&events, format)?;
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
            output_one(&resp.event, format)?;
        }
        EventCmd::Metadata { event_ticker } => {
            let path = format!("/events/{}/metadata", event_ticker);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}
