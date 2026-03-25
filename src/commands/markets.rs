use anyhow::Result;

use crate::cli::MarketCmd;
use crate::client::KalshiClient;
use crate::models::market::{
    CandlesticksResponse, MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
};
use crate::output::{OutputConfig, output, output_one, output_paginated, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

fn parse_fp(val: &Option<String>) -> f64 {
    val.as_deref()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0)
}

pub async fn execute(client: &KalshiClient, cmd: MarketCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        MarketCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            event_ticker,
        } => {
            let has_filter = series_ticker.is_some() || event_ticker.is_some();
            // When browsing unfiltered, fetch a big batch so we can sort by volume
            let effective_opts = if !has_filter && !all && limit.is_none() {
                PaginationOpts { limit: Some(1000), cursor: cursor.clone(), all: false }
            } else {
                PaginationOpts { limit, cursor, all }
            };
            let opts = effective_opts;
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
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

            // When no specific filter is applied, sort by volume so you
            // see active markets instead of random zero-volume junk.
            if !has_filter && !all {
                let display_limit = limit.unwrap_or(20) as usize;
                let mut markets = result.items;
                markets.sort_by(|a, b| {
                    let va = parse_fp(&a.volume_fp);
                    let vb = parse_fp(&b.volume_fp);
                    vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
                });
                markets.truncate(display_limit);
                output(&markets, out)?;
                eprintln!(
                    "Showing top {} markets by volume. Use --event-ticker or `market search` to filter.",
                    markets.len()
                );
            } else {
                output_paginated(&result.items, result.has_more, out)?;
            }
        }
        MarketCmd::Get { ticker } => {
            let path = format!("/markets/{}", ticker);
            let resp: MarketResponse = client.get(&path, &[]).await?;
            output_one(&resp.market, out)?;
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
                    let resp: TradesResponse = client.get("/markets/trades", &query_refs).await?;
                    Ok((resp.trades.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
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
            output(&resp.candlesticks.unwrap_or_default(), out)?;
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
            print_json(&resp.orderbook, out.no_pager)?;
        }
        MarketCmd::Search { query, limit, status } => {
            use crate::models::event::EventsResponse;

            let search_limit = limit.unwrap_or(20) as usize;
            let query_lower = query.to_lowercase();
            let mut found_markets = Vec::new();
            let mut cursor: Option<String> = None;

            // Search through events (much fewer than markets), then fetch
            // markets for matching events
            'outer: loop {
                let mut params = vec![("limit", "200".to_string())];
                if let Some(c) = cursor {
                    params.push(("cursor", c));
                }
                if let Some(ref s) = status {
                    params.push(("status", s.clone()));
                }
                let query_refs: Vec<(&str, &str)> =
                    params.iter().map(|(k, v)| (*k, v.as_str())).collect();
                let resp: EventsResponse = client.get("/events", &query_refs).await?;
                let events = resp.events.unwrap_or_default();
                let done = events.is_empty()
                    || resp.cursor.as_ref().map_or(true, |c| c.is_empty());

                for event in &events {
                    let title = event.title.as_deref().unwrap_or("").to_lowercase();
                    let ticker = event.event_ticker.as_deref().unwrap_or("").to_lowercase();
                    if title.contains(&query_lower) || ticker.contains(&query_lower) {
                        // Fetch markets for this event
                        let et = event.event_ticker.as_deref().unwrap_or("");
                        let mquery = [
                            ("event_ticker", et),
                            ("limit", "100"),
                        ];
                        let mresp: MarketsResponse =
                            client.get("/markets", &mquery).await?;
                        for m in mresp.markets.unwrap_or_default() {
                            found_markets.push(m);
                            if found_markets.len() >= search_limit {
                                break 'outer;
                            }
                        }
                    }
                }

                if done {
                    break;
                }
                cursor = resp.cursor;
            }

            if found_markets.is_empty() {
                println!("No markets found matching \"{}\".", query);
            } else {
                eprintln!("Found {} markets matching \"{}\".", found_markets.len(), query);
                output(&found_markets, out)?;
            }
        }
    }
    Ok(())
}
