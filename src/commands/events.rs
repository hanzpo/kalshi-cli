use anyhow::Result;

use crate::cli::EventCmd;
use crate::client::KalshiClient;
use crate::models::event::{EventResponse, EventsResponse};
use crate::output::{OutputConfig, output_one, output_paginated, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

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
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
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
            output_paginated(&result.items, result.has_more, out)?;
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
    }
    Ok(())
}
