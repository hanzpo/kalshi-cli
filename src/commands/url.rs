use anyhow::{Result, bail};
use std::process::Command;

use crate::client::KalshiClient;
use crate::models::event::EventResponse;

/// Split a ticker into (series_ticker, event_ticker).
///
/// Finds the first numeric segment to identify the boundary:
/// - "KXMARMAD-26-DUKE" → series="KXMARMAD", event="KXMARMAD-26"
/// - "KXMARMAD-26"      → series="KXMARMAD", event="KXMARMAD-26"
fn parse_ticker(ticker: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = ticker.split('-').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.chars().all(|c| c.is_ascii_digit()) && !part.is_empty() {
            let series = parts[..i].join("-");
            let event = parts[..=i].join("-");
            if series.is_empty() {
                return None;
            }
            return Some((series, event));
        }
    }
    None
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
    let (series_ticker, event_ticker) = parse_ticker(ticker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not parse ticker '{}' — expected format like KXMARMAD-26-DUKE",
            ticker
        )
    })?;

    // Fetch the event to get its sub_title for the URL slug
    let path = format!("/events/{}", event_ticker);
    let resp: EventResponse = client.get(&path, &[]).await?;
    let event = &resp.event;

    let sub_title = event
        .extra
        .get("sub_title")
        .and_then(|v| v.as_str())
        .or_else(|| event.title.as_deref());

    let slug = match sub_title {
        Some(title) => slugify(title),
        None => bail!(
            "Event '{}' has no title to build URL slug from",
            event_ticker
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
    fn parse_ticker_full_market() {
        let (series, event) = parse_ticker("KXMARMAD-26-DUKE").unwrap();
        assert_eq!(series, "KXMARMAD");
        assert_eq!(event, "KXMARMAD-26");
    }

    #[test]
    fn parse_ticker_event_only() {
        let (series, event) = parse_ticker("KXMARMAD-26").unwrap();
        assert_eq!(series, "KXMARMAD");
        assert_eq!(event, "KXMARMAD-26");
    }

    #[test]
    fn parse_ticker_multi_part_series() {
        let (series, event) = parse_ticker("INX-SPX-26MAR28-5720").unwrap();
        // First numeric segment: unclear — let's check
        // parts: ["INX", "SPX", "26MAR28", "5720"]
        // "26MAR28" is NOT all digits, "5720" IS
        assert_eq!(series, "INX-SPX-26MAR28");
        assert_eq!(event, "INX-SPX-26MAR28-5720");
    }

    #[test]
    fn parse_ticker_no_numeric() {
        assert!(parse_ticker("KXMARMAD").is_none());
    }

    #[test]
    fn parse_ticker_starts_with_number() {
        assert!(parse_ticker("26-FOO").is_none());
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
