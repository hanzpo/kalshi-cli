use anyhow::{Result, bail};

use crate::client::KalshiClient;
use crate::models::market::MarketsResponse;

/// Extract the event ticker from a Kalshi URL.
///
/// Expected format: https://kalshi.com/markets/{series}/{slug}/{event_ticker}
fn parse_url(url: &str) -> Option<String> {
    let path = url
        .strip_prefix("https://kalshi.com/")
        .or_else(|| url.strip_prefix("http://kalshi.com/"))
        .or_else(|| url.strip_prefix("https://www.kalshi.com/"))
        .or_else(|| url.strip_prefix("http://www.kalshi.com/"))?;

    let segments: Vec<&str> = path.trim_end_matches('/').split('/').collect();

    // Expected: ["markets", series, slug, event_ticker]
    if segments.len() >= 4 && segments[0] == "markets" {
        Some(segments[3].to_uppercase())
    } else {
        None
    }
}

pub async fn execute(client: &KalshiClient, url: &str) -> Result<()> {
    let event_ticker = parse_url(url).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not parse URL '{}' — expected format like https://kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26",
            url
        )
    })?;

    // Fetch markets for this event
    let path = "/markets";
    let resp: MarketsResponse = client
        .get(path, &[("event_ticker", &event_ticker), ("limit", "100")])
        .await?;

    let markets = resp.markets.unwrap_or_default();
    if markets.is_empty() {
        bail!("No markets found for event '{}'", event_ticker);
    }

    if markets.len() == 1 {
        println!(
            "{}",
            markets[0].ticker.as_deref().unwrap_or(&event_ticker)
        );
    } else {
        for market in &markets {
            if let Some(ticker) = &market.ticker {
                println!("{}", ticker);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_url() {
        assert_eq!(
            parse_url("https://kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26"),
            Some("KXMARMAD-26".to_string())
        );
    }

    #[test]
    fn parse_www_url() {
        assert_eq!(
            parse_url("https://www.kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26"),
            Some("KXMARMAD-26".to_string())
        );
    }

    #[test]
    fn parse_url_trailing_slash() {
        assert_eq!(
            parse_url("https://kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26/"),
            Some("KXMARMAD-26".to_string())
        );
    }

    #[test]
    fn parse_url_bad_format() {
        assert!(parse_url("https://kalshi.com/events/foo").is_none());
    }

    #[test]
    fn parse_url_not_kalshi() {
        assert!(parse_url("https://example.com/markets/a/b/c").is_none());
    }
}
