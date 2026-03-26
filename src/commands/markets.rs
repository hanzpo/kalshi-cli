use anyhow::Result;

use crate::browse;
use crate::cli::MarketCmd;
use crate::client::KalshiClient;
use crate::models::market::{
    CandlesticksResponse, MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
};
use crate::output::{OutputConfig, output, output_one, output_paginated, print_json};
use crate::pagination::{DEFAULT_DISPLAY_LIMIT, MARKETS_PAGE_SIZE, PaginationOpts, auto_paginate, paginated_list};

/// Maximum number of markets to scan for aggregate commands (hot, expiring, spread).
/// With 1000/page and 200ms throttle, 20000 = 20 requests ≈ 4 seconds.
const SCAN_LIMIT: u32 = 20000;

/// Allowed page sizes for the internal search API.
const SEARCH_PAGE_SIZES: &[u32] = &[3, 5, 8, 25, 30, 50, 70, 100];

/// Pick the nearest allowed page_size for the search API.
fn pick_search_page_size(desired: u32) -> u32 {
    *SEARCH_PAGE_SIZES
        .iter()
        .find(|&&s| s >= desired)
        .unwrap_or(&100)
}

/// Build the query params shared by the paginated fetcher.
fn build_market_query(
    page_limit: u32,
    page_cursor: Option<String>,
    status: &Option<String>,
    series_ticker: &Option<String>,
    event_ticker: &Option<String>,
    include_combos: bool,
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
    if !include_combos {
        query.push(("mve_filter".to_string(), "exclude".to_string()));
    }
    query
}

async fn fetch_markets_page(
    client: &KalshiClient,
    query: &[(String, String)],
) -> Result<(Vec<crate::models::market::Market>, Option<String>)> {
    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let resp: MarketsResponse = client.get("/markets", &query_refs).await?;
    Ok((resp.markets.unwrap_or_default(), resp.cursor))
}


pub async fn execute(client: &KalshiClient, cmd: MarketCmd, format: &OutputConfig) -> Result<()> {
    match cmd {
        MarketCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            event_ticker,
            include_combos,
        } => {
            paginated_list(all, limit, cursor, Some(MARKETS_PAGE_SIZE), format, |page_limit, page_cursor| {
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
                        include_combos,
                    );
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;
        }
        MarketCmd::Get { ticker } => {
            let path = format!("/markets/{}", ticker);
            let resp: MarketResponse = client.get(&path, &[]).await?;
            output_one(&resp.market, format)?;
        }
        MarketCmd::Trade {
            ticker,
            limit,
            cursor,
            all,
            min_ts,
            max_ts,
        } => {
            paginated_list(all, limit, cursor, Some(MARKETS_PAGE_SIZE), format, |page_limit: u32, page_cursor: Option<String>| {
                let ticker = ticker.clone();
                let min_ts = min_ts;
                let max_ts = max_ts;
                async move {
                    let mut query: Vec<(String, String)> =
                        vec![("limit".to_string(), page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor".to_string(), c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker".to_string(), t.clone()));
                    }
                    if let Some(ts) = min_ts {
                        query.push(("min_ts".to_string(), ts.to_string()));
                    }
                    if let Some(ts) = max_ts {
                        query.push(("max_ts".to_string(), ts.to_string()));
                    }
                    let query_refs: Vec<(&str, &str)> = query
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    let resp: TradesResponse = client.get("/markets/trades", &query_refs).await?;
                    Ok((resp.trades.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
        }
        MarketCmd::Candlestick {
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
            output_paginated(&resp.candlesticks.unwrap_or_default(), false, format)?;
        }
        MarketCmd::Search {
            query,
            limit,
            cursor,
            all,
            status: _,
            include_combos: _,
        } => {
            use crate::models::market::SearchResponse;

            // Use Kalshi's internal semantic search API.
            let search_url = format!("{}/v1/search/series", client.host());
            // Map our limit to nearest allowed page_size: 3, 5, 8, 25, 30, 50, 70, 100
            let page_size = pick_search_page_size(limit.unwrap_or(DEFAULT_DISPLAY_LIMIT));
            let page_size_str = page_size.to_string();

            if all {
                // Interactive browser
                let display_size = limit.unwrap_or(25);
                let ps = pick_search_page_size(display_size).to_string();
                let initial_cursor = cursor;
                browse::browse(display_size, |_display_limit, page_cursor| {
                    let search_url = search_url.clone();
                    let query = query.clone();
                    let ps = ps.clone();
                    let initial_cursor = initial_cursor.clone();
                    async move {
                        let effective_cursor = page_cursor.or(initial_cursor);
                        let mut params: Vec<(&str, &str)> = vec![
                            ("query", query.as_str()),
                            ("order_by", "querymatch"),
                            ("page_size", ps.as_str()),
                            ("fuzzy_threshold", "4"),
                        ];
                        let cursor_val;
                        if let Some(ref c) = effective_cursor {
                            cursor_val = c.clone();
                            params.push(("cursor", &cursor_val));
                        }
                        let resp: SearchResponse =
                            client.get_absolute(&search_url, &params).await?;
                        let items = resp.current_page.unwrap_or_default();
                        Ok((items, resp.next_cursor))
                    }
                })
                .await?;
            } else {
                let mut params: Vec<(&str, &str)> = vec![
                    ("query", query.as_str()),
                    ("order_by", "querymatch"),
                    ("page_size", &page_size_str),
                    ("fuzzy_threshold", "4"),
                ];
                let cursor_val;
                if let Some(ref c) = cursor {
                    cursor_val = c.clone();
                    params.push(("cursor", &cursor_val));
                }
                let resp: SearchResponse =
                    client.get_absolute(&search_url, &params).await?;
                let items = resp.current_page.unwrap_or_default();
                let has_more = resp.next_cursor.as_ref().is_some_and(|c| !c.is_empty());
                if let Some(total) = resp.total_results_count {
                    eprintln!("{} results found", total);
                }
                output_paginated(&items, has_more, format)?;
            }
        }
        MarketCmd::CandlestickBatch {
            tickers,
            start_ts,
            end_ts,
            period,
        } => {
            let mut query = vec![("market_tickers", tickers.as_str())];
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
            let resp: serde_json::Value = client.get("/markets/candlesticks", &query).await?;
            print_json(&resp, format.no_pager)?;
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
            print_json(&resp.orderbook, format.no_pager)?;
        }
        MarketCmd::Hot { limit: top_n, include_combos } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(page_limit, page_cursor, &status, &None, &None, include_combos);
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let mut markets = result.items;
            markets.sort_by(|a, b| b.volume_24h.unwrap_or(0).cmp(&a.volume_24h.unwrap_or(0)));
            markets.truncate(top_n as usize);
            if result.has_more {
                eprintln!("(scanned {} markets; results are approximate)", SCAN_LIMIT);
            }
            output(&markets, format)?;
        }
        MarketCmd::Expiring { within, include_combos } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(page_limit, page_cursor, &status, &None, &None, include_combos);
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let now = chrono::Utc::now();
            let cutoff = now + chrono::Duration::hours(within as i64);

            let mut expiring: Vec<_> = result
                .items
                .into_iter()
                .filter(|m| {
                    if let Some(ref ct) = m.close_time {
                        if let Ok(t) = chrono::DateTime::parse_from_rfc3339(ct) {
                            let t_utc = t.with_timezone(&chrono::Utc);
                            return t_utc > now && t_utc <= cutoff;
                        }
                    }
                    false
                })
                .collect();
            expiring.sort_by(|a, b| a.close_time.cmp(&b.close_time));
            if result.has_more {
                eprintln!(
                    "(scanned {} markets; some expiring markets may be missing)",
                    SCAN_LIMIT
                );
            }
            output(&expiring, format)?;
        }
        MarketCmd::Spread { limit: top_n, include_combos } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(page_limit, page_cursor, &status, &None, &None, include_combos);
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let mut markets: Vec<_> = result
                .items
                .into_iter()
                .filter(|m| m.yes_bid.is_some() && m.yes_ask.is_some())
                .collect();
            markets.sort_by(|a, b| {
                let spread_a = a.yes_ask.unwrap_or(0.0) - a.yes_bid.unwrap_or(0.0);
                let spread_b = b.yes_ask.unwrap_or(0.0) - b.yes_bid.unwrap_or(0.0);
                spread_b
                    .partial_cmp(&spread_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            markets.truncate(top_n as usize);
            if result.has_more {
                eprintln!("(scanned {} markets; results are approximate)", SCAN_LIMIT);
            }
            output(&markets, format)?;
        }
        MarketCmd::Analyze { ticker, buy, sell } => {
            client.require_auth()?;
            let path = format!("/markets/{}/orderbook", ticker);
            let resp: OrderbookResponse = client.get(&path, &[("depth", "100")]).await?;
            let ob = resp
                .orderbook
                .ok_or_else(|| anyhow::anyhow!("No orderbook data returned"))?;

            let yes_levels = ob.yes.unwrap_or_default();
            let no_levels = ob.no.unwrap_or_default();

            let parse_levels = |levels: &[Vec<serde_json::Value>]| -> Vec<(f64, i64)> {
                levels
                    .iter()
                    .filter_map(|row| {
                        if row.len() >= 2 {
                            let price = row[0].as_f64()?;
                            let qty = row[1].as_i64()?;
                            Some((price, qty))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            let yes_parsed = parse_levels(&yes_levels);
            let no_parsed = parse_levels(&no_levels);

            let best_bid = yes_parsed.last().map(|(p, _)| *p);
            let best_ask = yes_parsed.first().map(|(p, _)| *p);
            let spread = match (best_bid, best_ask) {
                (Some(bid), Some(ask)) => Some(ask - bid),
                _ => None,
            };

            let total_yes_depth: i64 = yes_parsed.iter().map(|(_, q)| *q).sum();
            let total_no_depth: i64 = no_parsed.iter().map(|(_, q)| *q).sum();

            println!("=== Orderbook Analysis: {} ===", ticker);
            println!();
            println!(
                "Best Bid:       {}",
                best_bid.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!(
                "Best Ask:       {}",
                best_ask.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!(
                "Spread:         {}",
                spread.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!("Yes-side depth: {} contracts", total_yes_depth);
            println!("No-side depth:  {} contracts", total_no_depth);

            // Simulate fill
            if let Some(qty) = buy {
                let cost = simulate_fill(&yes_parsed, qty, true);
                println!();
                println!(
                    "Simulated BUY {} contracts: avg price {:.2}c, total cost {:.2}c",
                    qty,
                    if qty > 0 { cost / qty as f64 } else { 0.0 },
                    cost
                );
            }
            if let Some(qty) = sell {
                let proceeds = simulate_fill(&yes_parsed, qty, false);
                println!();
                println!(
                    "Simulated SELL {} contracts: avg price {:.2}c, total proceeds {:.2}c",
                    qty,
                    if qty > 0 { proceeds / qty as f64 } else { 0.0 },
                    proceeds
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn simulate_fill(levels: &[(f64, i64)], mut qty: i64, buying: bool) -> f64 {
    let mut total_cost = 0.0;
    // For buying, walk the ask side (ascending price); for selling, walk the bid side (descending)
    let iter: Box<dyn Iterator<Item = &(f64, i64)>> = if buying {
        Box::new(levels.iter())
    } else {
        Box::new(levels.iter().rev())
    };
    for &(price, available) in iter {
        if qty <= 0 {
            break;
        }
        let fill = qty.min(available);
        total_cost += price * fill as f64;
        qty -= fill;
    }
    total_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── simulate_fill tests ──

    #[test]
    fn simulate_fill_buy_across_multiple_levels() {
        let levels = vec![(10.0, 5), (20.0, 10), (30.0, 5)];
        // Buy 8: fills 5@10 + 3@20 = 50 + 60 = 110
        let cost = simulate_fill(&levels, 8, true);
        assert!((cost - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_empty_levels() {
        let levels: Vec<(f64, i64)> = vec![];
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_more_than_available() {
        let levels = vec![(10.0, 3), (20.0, 2)];
        // Want 10, only 5 available: 3@10 + 2@20 = 30 + 40 = 70
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_sell_reverses_order() {
        let levels = vec![(10.0, 5), (20.0, 10), (30.0, 5)];
        // Sell 8: walks from end → 5@30 + 3@20 = 150 + 60 = 210
        let proceeds = simulate_fill(&levels, 8, false);
        assert!((proceeds - 210.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_exactly_one_level() {
        let levels = vec![(15.0, 7), (25.0, 3)];
        // Buy exactly 7 → fills entire first level: 7@15 = 105
        let cost = simulate_fill(&levels, 7, true);
        assert!((cost - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_single_level_partial() {
        let levels = vec![(42.0, 100)];
        // Buy 10 from single level: 10@42 = 420
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 420.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_zero_quantity() {
        let levels = vec![(10.0, 5)];
        let cost = simulate_fill(&levels, 0, true);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_sell_empty_levels() {
        let levels: Vec<(f64, i64)> = vec![];
        let proceeds = simulate_fill(&levels, 5, false);
        assert!((proceeds - 0.0).abs() < f64::EPSILON);
    }

    // ── build_market_query tests ──

    #[test]
    fn build_market_query_excludes_combos_by_default() {
        let query = build_market_query(100, None, &None, &None, &None, false);
        assert_eq!(query.len(), 2);
        assert_eq!(query[0], ("limit".to_string(), "100".to_string()));
        assert_eq!(query[1], ("mve_filter".to_string(), "exclude".to_string()));
    }

    #[test]
    fn build_market_query_include_combos() {
        let query = build_market_query(100, None, &None, &None, &None, true);
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("limit".to_string(), "100".to_string()));
    }

    #[test]
    fn build_market_query_with_cursor() {
        let query = build_market_query(50, Some("abc123".to_string()), &None, &None, &None, false);
        assert_eq!(query.len(), 3);
        assert_eq!(query[1], ("cursor".to_string(), "abc123".to_string()));
    }

    #[test]
    fn build_market_query_with_all_params() {
        let status = Some("open".to_string());
        let series = Some("SER-1".to_string());
        let event = Some("EVT-1".to_string());
        let query = build_market_query(25, Some("cur".to_string()), &status, &series, &event, false);
        assert_eq!(query.len(), 6);
        assert_eq!(query[0].0, "limit");
        assert_eq!(query[1], ("cursor".to_string(), "cur".to_string()));
        assert_eq!(query[2], ("status".to_string(), "open".to_string()));
        assert_eq!(query[3], ("series_ticker".to_string(), "SER-1".to_string()));
        assert_eq!(query[4], ("event_ticker".to_string(), "EVT-1".to_string()));
        assert_eq!(query[5], ("mve_filter".to_string(), "exclude".to_string()));
    }

    #[test]
    fn build_market_query_with_status_only() {
        let status = Some("closed".to_string());
        let query = build_market_query(10, None, &status, &None, &None, false);
        assert_eq!(query.len(), 3);
        assert_eq!(query[1], ("status".to_string(), "closed".to_string()));
    }

    #[test]
    fn build_market_query_with_event_ticker_only() {
        let event = Some("EVT-ABC".to_string());
        let query = build_market_query(10, None, &None, &None, &event, false);
        assert_eq!(query.len(), 3);
        assert_eq!(
            query[1],
            ("event_ticker".to_string(), "EVT-ABC".to_string())
        );
    }
}
