use anyhow::Result;

use crate::browse;
use crate::cli::MarketCmd;
use crate::client::KalshiClient;
use crate::models::market::{
    CandlesticksResponse, MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
};
use crate::output::{OutputFormat, output, output_one, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

/// Build the query params shared by the paginated fetcher.
fn build_market_query(
    page_limit: u32,
    page_cursor: Option<String>,
    status: &Option<String>,
    series_ticker: &Option<String>,
    event_ticker: &Option<String>,
) -> Vec<(String, String)> {
    let mut query = vec![("limit".to_string(), page_limit.to_string())];
    if let Some(c) = page_cursor {
        query.push(("cursor".to_string(), c));
    }
    if let Some(s) = status {
        query.push(("status".to_string(), s.clone()));
    }
    if let Some(s) = series_ticker {
        query.push(("series_ticker".to_string(), s.clone()));
    }
    if let Some(e) = event_ticker {
        query.push(("event_ticker".to_string(), e.clone()));
    }
    query
}

async fn fetch_markets_page(
    client: &KalshiClient,
    query: &[(String, String)],
) -> Result<(Vec<crate::models::market::Market>, Option<String>)> {
    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let resp: MarketsResponse = client.get("/markets", &query_refs).await?;
    Ok((resp.markets.unwrap_or_default(), resp.cursor))
}

pub async fn execute(client: &KalshiClient, cmd: MarketCmd, format: &OutputFormat) -> Result<()> {
    match cmd {
        MarketCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            event_ticker,
        } => {
            if all {
                // Interactive paginated browser — fetches one page at a time.
                let page_size: u32 = limit.unwrap_or(50);
                let initial_cursor = cursor;
                browse::browse(page_size, |page_limit, page_cursor| {
                    let status = status.clone();
                    let series_ticker = series_ticker.clone();
                    let event_ticker = event_ticker.clone();
                    let initial_cursor = initial_cursor.clone();
                    async move {
                        // On the very first call page_cursor is None; use --cursor if provided.
                        let effective_cursor = page_cursor.or(initial_cursor);
                        let query = build_market_query(
                            page_limit,
                            effective_cursor,
                            &status,
                            &series_ticker,
                            &event_ticker,
                        );
                        fetch_markets_page(client, &query).await
                    }
                })
                .await?;
            } else {
                let opts = PaginationOpts { limit, cursor, all };
                let markets = auto_paginate(&opts, 100, |page_limit, page_cursor| {
                    let status = status.clone();
                    let series_ticker = series_ticker.clone();
                    let event_ticker = event_ticker.clone();
                    async move {
                        let query = build_market_query(
                            page_limit,
                            page_cursor,
                            &status,
                            &series_ticker,
                            &event_ticker,
                        );
                        fetch_markets_page(client, &query).await
                    }
                })
                .await?;
                output(&markets, format)?;
            }
        }
        MarketCmd::Get { ticker } => {
            let path = format!("/markets/{}", ticker);
            let resp: MarketResponse = client.get(&path, &[]).await?;
            output_one(&resp.market, format)?;
        }
        MarketCmd::Trades {
            ticker,
            limit,
            cursor,
            all,
            min_ts,
            max_ts,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let trades = auto_paginate(&opts, 100, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                let min_ts = min_ts;
                let max_ts = max_ts;
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
                    let resp: TradesResponse = client.get("/markets/trades", &query_refs).await?;
                    Ok((resp.trades.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output(&trades, format)?;
        }
        MarketCmd::Candlesticks {
            ticker,
            series_ticker,
            period,
            start_ts,
            end_ts,
        } => {
            let path = format!("/series/{}/markets/{}/candlesticks", series_ticker, ticker);
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
            output(&resp.candlesticks.unwrap_or_default(), format)?;
        }
        MarketCmd::Orderbook { ticker, depth } => {
            client.require_auth()?;
            let path = format!("/markets/{}/orderbook", ticker);
            let depth_str = depth.map(|d| d.to_string());
            let mut query = Vec::new();
            if let Some(ref d) = depth_str {
                query.push(("depth", d.as_str()));
            }
            let resp: OrderbookResponse = client.get(&path, &query).await?;
            // Orderbook is complex nested data, always print as JSON
            print_json(&resp.orderbook)?;
        }
    }
    Ok(())
}
