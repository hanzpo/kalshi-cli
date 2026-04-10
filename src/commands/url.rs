use anyhow::{Context, Result};
use std::io::{self, IsTerminal};
use std::process::Command;

use crate::client::KalshiClient;
use crate::error::KalshiError;
use crate::models::event::{Event, EventResponse};
use crate::models::market::MarketResponse;
use crate::models::series::SeriesResponse;

/// Convert a string into a URL-safe slug: lowercase, strip special chars, whitespace → hyphens.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn build_event_url(series_ticker: &str, series_title: &str, event_ticker: &str) -> String {
    format!(
        "https://kalshi.com/markets/{}/{}/{}",
        series_ticker.to_lowercase(),
        slugify(series_title),
        event_ticker.to_lowercase(),
    )
}

fn format_terminal_hyperlink(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

async fn resolve_event(client: &KalshiClient, ticker: &str) -> Result<Event> {
    let upper = ticker.to_uppercase();
    let market_path = format!("/markets/{upper}");

    match client.get::<MarketResponse>(&market_path, &[]).await {
        Ok(resp) => {
            let event_ticker = resp.market.event_ticker.unwrap_or(upper);
            let path = format!("/events/{}", event_ticker.to_uppercase());
            let event_resp: EventResponse = client
                .get(&path, &[])
                .await
                .with_context(|| format!("Failed to resolve event for market via {path}"))?;
            Ok(event_resp.event)
        }
        Err(e) => {
            if let Some(KalshiError::Api { status: 404, .. }) = e.downcast_ref::<KalshiError>() {
                let path = format!("/events/{upper}");
                let event_resp: EventResponse = client
                    .get(&path, &[])
                    .await
                    .with_context(|| {
                        format!(
                            "Ticker '{ticker}' was not found as a market via {market_path}, and also not found as an event via {path}"
                        )
                    })?;
                Ok(event_resp.event)
            } else {
                Err(e).with_context(|| format!("Failed to fetch market via {market_path}"))
            }
        }
    }
}

pub async fn execute(client: &KalshiClient, ticker: &str, open: bool) -> Result<()> {
    let event = resolve_event(client, ticker).await?;
    let series_ticker = event
        .series_ticker
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Event is missing series_ticker"))?;
    let event_ticker = event
        .event_ticker
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Event is missing event_ticker"))?;
    let series_path = format!("/series/{}", series_ticker.to_uppercase());
    let series_resp: SeriesResponse = client
        .get(&series_path, &[])
        .await
        .with_context(|| format!("Failed to fetch series via {series_path}"))?;
    let series_title = series_resp
        .series
        .title
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Series '{}' has no title", series_ticker))?;

    let url = build_event_url(series_ticker, series_title, event_ticker);

    if io::stdout().is_terminal() {
        println!("{}", format_terminal_hyperlink(&url, &url));
    } else {
        println!("{}", url);
    }

    if open {
        #[cfg(target_os = "macos")]
        Command::new("open").arg(&url).spawn()?;
        #[cfg(target_os = "linux")]
        Command::new("xdg-open").arg(&url).spawn()?;
        #[cfg(target_os = "windows")]
        Command::new("cmd").args(["/C", "start", &url]).spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_event_url_uses_series_title_slug() {
        assert_eq!(
            build_event_url("KXPGATOUR", "PGA Tour", "KXPGATOUR-MAST26"),
            "https://kalshi.com/markets/kxpgatour/pga-tour/kxpgatour-mast26"
        );
    }

    #[test]
    fn format_terminal_hyperlink_wraps_url() {
        assert_eq!(
            format_terminal_hyperlink("https://kalshi.com", "https://kalshi.com"),
            "\x1b]8;;https://kalshi.com\x1b\\https://kalshi.com\x1b]8;;\x1b\\"
        );
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("March Madness 2026"), "march-madness-2026");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("Who's #1? The Best!"), "whos-1-the-best");
    }

    #[test]
    fn slugify_extra_whitespace() {
        assert_eq!(slugify("  hello   world  "), "hello-world");
    }
}
