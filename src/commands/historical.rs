use anyhow::Result;

use crate::cli::HistoricalCmd;
use crate::client::KalshiClient;
use crate::models::historical::{CutoffResponse, HistoricalMarketsResponse};
use crate::models::market::{CandlesticksResponse, TradesResponse};
use crate::models::order::OrdersResponse;
use crate::models::portfolio::FillsResponse;
use crate::output::{OutputConfig, output, output_paginated, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

pub async fn execute(
    client: &KalshiClient,
    cmd: HistoricalCmd,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        HistoricalCmd::Market {
            limit,
            cursor,
            ticker,
            min_close_ts,
            max_close_ts,
        } => {
            let opts = PaginationOpts {
                limit,
                cursor,
                all: false,
            };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    if let Some(ts) = min_close_ts {
                        query.push(("min_close_ts", ts.to_string()));
                    }
                    if let Some(ts) = max_close_ts {
                        query.push(("max_close_ts", ts.to_string()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: HistoricalMarketsResponse =
                        client.get("/historical/markets", &query_refs).await?;
                    Ok((resp.markets.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        HistoricalCmd::Trade {
            limit,
            cursor,
            ticker,
            min_ts,
            max_ts,
        } => {
            let opts = PaginationOpts {
                limit,
                cursor,
                all: false,
            };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    if let Some(ts) = min_ts {
                        query.push(("min_ts", ts.to_string()));
                    }
                    if let Some(ts) = max_ts {
                        query.push(("max_ts", ts.to_string()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: TradesResponse =
                        client.get("/historical/trades", &query_refs).await?;
                    Ok((resp.trades.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        HistoricalCmd::Candlestick {
            ticker,
            series_ticker,
            period,
            start_ts,
            end_ts,
        } => {
            let path = format!("/historical/markets/{}/candlesticks", ticker);
            let mut query = Vec::new();
            let series_str = series_ticker;
            let period_str = period.map(|p| p.to_string());
            let start_str = start_ts.map(|t| t.to_string());
            let end_str = end_ts.map(|t| t.to_string());

            query.push(("series_ticker", series_str.as_str()));
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
        HistoricalCmd::Cutoff => {
            let resp: CutoffResponse = client.get("/historical/cutoff", &[]).await?;
            println!(
                "Cutoff timestamp: {}",
                resp.cutoff_ts.unwrap_or_else(|| "-".to_string())
            );
        }
        HistoricalCmd::Fill {
            limit,
            cursor,
            ticker,
        } => {
            client.require_auth()?;
            let opts = PaginationOpts {
                limit,
                cursor,
                all: false,
            };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: FillsResponse =
                        client.get("/historical/fills", &query_refs).await?;
                    Ok((resp.fills.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        HistoricalCmd::Order {
            limit,
            cursor,
            ticker,
        } => {
            client.require_auth()?;
            let opts = PaginationOpts {
                limit,
                cursor,
                all: false,
            };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: OrdersResponse =
                        client.get("/historical/orders", &query_refs).await?;
                    Ok((resp.orders.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        HistoricalCmd::MarketDetail { ticker } => {
            let path = format!("/historical/markets/{}", ticker);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
