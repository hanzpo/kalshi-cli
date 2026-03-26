use anyhow::Result;

use crate::cli::SeriesCmd;
use crate::client::KalshiClient;
use crate::models::series::{SeriesListResponse, SeriesResponse};
use crate::output::{OutputConfig, output_one, output_paginated, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

pub async fn execute(client: &KalshiClient, cmd: SeriesCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        SeriesCmd::List { limit, cursor, all } => {
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| async move {
                let mut query = vec![("limit", page_limit.to_string())];
                if let Some(c) = page_cursor {
                    query.push(("cursor", c));
                }
                let query_refs: Vec<(&str, &str)> =
                    query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                let resp: SeriesListResponse = client.get("/series", &query_refs).await?;
                Ok((resp.series.unwrap_or_default(), resp.cursor))
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        SeriesCmd::Get { series_ticker } => {
            let path = format!("/series/{}", series_ticker);
            let resp: SeriesResponse = client.get(&path, &[]).await?;
            output_one(&resp.series, out)?;
        }
        SeriesCmd::FeeChange { series_ticker, show_historical } => {
            let mut query = Vec::new();
            if let Some(ref s) = series_ticker {
                query.push(("series_ticker", s.as_str()));
            }
            if show_historical {
                query.push(("show_historical", "true"));
            }
            let resp: serde_json::Value = client.get("/series/fee_changes", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
