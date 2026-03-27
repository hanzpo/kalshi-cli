use anyhow::{Result, bail};
use std::process::Command;

use crate::client::KalshiClient;
use crate::error::KalshiError;
use crate::models::market::MarketResponse;
use crate::models::series::SeriesResponse;

/// Extract the series ticker from a full ticker.
///
/// The series is everything before the first segment that starts with a digit:
/// - "KXMARMAD-26-DUKE"              → "KXMARMAD"
/// - "KXMLBGAME-26MAR271907ATHTOR"   → "KXMLBGAME"
/// - "INX-SPX-26MAR28-5720"          → "INX-SPX"
fn parse_series(ticker: &str) -> Option<String> {
    if ticker.is_empty() {
        return None;
    }
    let parts: Vec<&str> = ticker.split('-').collect();
    for (i, part) in parts.iter().enumerate() {
        if !part.is_empty() && part.starts_with(|c: char| c.is_ascii_digit()) {
            let series = parts[..i].join("-");
            if series.is_empty() {
                return None;
            }
            return Some(series);
        }
    }
    // No numeric segment found — the whole ticker is the series (e.g. "KXCOMPANYACTIONLAYOFF")
    Some(ticker.to_string())
}

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

pub async fn execute(client: &KalshiClient, ticker: &str, open: bool) -> Result<()> {
    // Try fetching as a market ticker first; if 404, treat input as an event ticker
    let event_ticker = match client
        .get::<MarketResponse>(&format!("/markets/{}", ticker), &[])
        .await
    {
        Ok(resp) => resp
            .market
            .event_ticker
            .unwrap_or_else(|| ticker.to_string()),
        Err(e) => {
            if let Some(KalshiError::Api { status: 404, .. }) = e.downcast_ref::<KalshiError>() {
                ticker.to_string()
            } else {
                return Err(e);
            }
        }
    };

    let series_ticker = parse_series(&event_ticker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not parse ticker '{}' — expected format like KXMARMAD-26",
            event_ticker
        )
    })?;

    // Fetch the series to get its title for the URL slug
    let series_path = format!("/series/{}", series_ticker);
    let series_resp: SeriesResponse = client.get(&series_path, &[]).await?;
    let series = &series_resp.series;

    let slug = match series.title.as_deref() {
        Some(title) => slugify(title),
        None => bail!(
            "Series '{}' has no title to build URL slug from",
            series_ticker
        ),
    };

    let url = format!(
        "https://kalshi.com/markets/{}/{}/{}",
        series_ticker.to_lowercase(),
        slug,
        event_ticker.to_lowercase(),
    );

    println!("{}", url);

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
    fn parse_series_full_market() {
        assert_eq!(parse_series("KXMARMAD-26-DUKE").unwrap(), "KXMARMAD");
    }

    #[test]
    fn parse_series_event_only() {
        assert_eq!(parse_series("KXMARMAD-26").unwrap(), "KXMARMAD");
    }

    #[test]
    fn parse_series_multi_part() {
        assert_eq!(parse_series("INX-SPX-26MAR28-5720").unwrap(), "INX-SPX");
    }

    #[test]
    fn parse_series_no_pure_numeric_segment() {
        assert_eq!(
            parse_series("KXMLBGAME-26MAR271907ATHTOR").unwrap(),
            "KXMLBGAME"
        );
    }

    #[test]
    fn parse_series_no_numeric() {
        assert_eq!(parse_series("KXCOMPANYACTIONLAYOFF").unwrap(), "KXCOMPANYACTIONLAYOFF");
    }

    #[test]
    fn parse_series_empty() {
        assert!(parse_series("").is_none());
    }

    #[test]
    fn parse_series_starts_with_number() {
        assert!(parse_series("26-FOO").is_none());
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
