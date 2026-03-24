use anyhow::Result;

use crate::cli::MarketCmd;
use crate::client::KalshiClient;
use crate::models::market::{
    CandlesticksResponse, MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
};
use crate::output::{OutputFormat, output, output_one, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

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
            let opts = PaginationOpts { limit, cursor, all };
            let markets = auto_paginate(&opts, 100, |page_limit, page_cursor| {
                let status = status.clone();
                let series_ticker = series_ticker.clone();
                let event_ticker = event_ticker.clone();
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
                    if let Some(ref e) = event_ticker {
                        query.push(("event_ticker", e.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: MarketsResponse = client.get("/markets", &query_refs).await?;
                    Ok((resp.markets.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output(&markets, format)?;
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
