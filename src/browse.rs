use std::future::Future;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use crate::error::KalshiError;
use crate::output::TableDisplay;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 1000;

struct Page<T> {
    items: Vec<T>,
    /// Cursor to fetch the *next* page after this one.
    next_cursor: Option<String>,
}

/// Fetch with automatic retry on 429 rate-limit responses.
/// Uses exponential backoff starting at 1s (1s, 2s, 4s, 8s, 16s).
/// Shows a countdown in the terminal while waiting.
async fn fetch_with_retry<T, F, Fut>(
    fetcher: &F,
    page_size: u32,
    cursor: Option<String>,
) -> Result<(Vec<T>, Option<String>)>
where
    F: Fn(u32, Option<String>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>>,
{
    let mut backoff_ms = INITIAL_BACKOFF_MS;

    for attempt in 0..=MAX_RETRIES {
        match fetcher(page_size, cursor.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if let Some(KalshiError::Api { status: 429, .. }) = e.downcast_ref::<KalshiError>()
                {
                    if attempt == MAX_RETRIES {
                        return Err(e);
                    }
                    show_retry_countdown(backoff_ms, attempt + 1).await;
                    backoff_ms *= 2;
                } else {
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}

/// Display a countdown timer while waiting for rate limit to clear.
async fn show_retry_countdown(total_ms: u64, attempt: u32) {
    let total_secs = (total_ms + 999) / 1000; // round up
    for remaining in (1..=total_secs).rev() {
        // \r overwrites the current line; raw mode is off here
        print!(
            "\r  Rate limited — retrying in {remaining}s (attempt {attempt}/{MAX_RETRIES})...  "
        );
        let _ = io::stdout().flush();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    // Clear the status line
    print!("\r{}\r", " ".repeat(70));
    let _ = io::stdout().flush();
}

/// Interactive paginated browser.
///
/// Fetches one page at a time and lets the user navigate with keyboard controls.
/// Previously fetched pages are cached so going back is free.
/// Automatically retries with backoff on 429 rate-limit errors.
pub async fn browse<T, F, Fut>(page_size: u32, fetcher: F) -> Result<()>
where
    T: TableDisplay,
    F: Fn(u32, Option<String>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>>,
{
    // Fetch the first page.
    let (items, next_cursor) = fetch_with_retry(&fetcher, page_size, None).await?;
    if items.is_empty() {
        println!("No results.");
        return Ok(());
    }

    let mut pages: Vec<Page<T>> = vec![Page { items, next_cursor }];
    let mut current: usize = 0;
    let mut status_msg: Option<String> = None;

    loop {
        render_page(&pages[current], current, pages.len(), &status_msg);
        status_msg = None;

        match read_key()? {
            Action::NextPage => {
                let next_index = current + 1;
                if next_index < pages.len() {
                    current = next_index;
                } else {
                    let cursor = pages[current].next_cursor.clone();
                    if cursor.as_ref().is_some_and(|c| !c.is_empty()) {
                        // Show loading indicator before fetch.
                        print!("\r  Loading next page...");
                        let _ = io::stdout().flush();

                        match fetch_with_retry(&fetcher, page_size, cursor).await {
                            Ok((items, next_cursor)) => {
                                if !items.is_empty() {
                                    pages.push(Page { items, next_cursor });
                                    current = next_index;
                                } else {
                                    status_msg = Some("No more results.".to_string());
                                }
                            }
                            Err(e) => {
                                status_msg = Some(format!("Error: {e}"));
                            }
                        }
                    } else {
                        status_msg = Some("Last page — no more results.".to_string());
                    }
                }
            }
            Action::PrevPage => {
                if current > 0 {
                    current -= 1;
                }
            }
            Action::Quit => break,
        }
    }

    Ok(())
}

fn render_page<T: TableDisplay>(
    page: &Page<T>,
    page_idx: usize,
    total_cached: usize,
    status_msg: &Option<String>,
) {
    // Clear screen and move cursor to top.
    print!("\x1B[2J\x1B[H");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(T::headers());
    for item in &page.items {
        table.add_row(item.row());
    }
    println!("{table}");

    let has_next = page.next_cursor.as_ref().is_some_and(|c| !c.is_empty());
    let page_num = page_idx + 1;

    println!();
    println!(
        "  Page {page_num}{cached}  |  {items} items  |  {nav}",
        cached = if total_cached > page_num {
            format!(" ({}+ cached)", total_cached)
        } else {
            String::new()
        },
        items = page.items.len(),
        nav = build_nav(page_idx > 0, has_next),
    );

    if let Some(msg) = status_msg {
        println!("  {msg}");
    }

    let _ = io::stdout().flush();
}

fn build_nav(has_prev: bool, has_next: bool) -> String {
    let mut parts = Vec::new();
    if has_prev {
        parts.push("[p/\u{2190}] Prev");
    }
    if has_next {
        parts.push("[n/\u{2192}] Next");
    }
    parts.push("[q] Quit");
    parts.join("  ")
}

enum Action {
    NextPage,
    PrevPage,
    Quit,
}

fn read_key() -> Result<Action> {
    terminal::enable_raw_mode()?;
    let result = read_key_inner();
    terminal::disable_raw_mode()?;
    result
}

fn read_key_inner() -> Result<Action> {
    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('n') | KeyCode::Right => return Ok(Action::NextPage),
                KeyCode::Char('p') | KeyCode::Left => return Ok(Action::PrevPage),
                KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::Quit),
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(Action::Quit);
                }
                _ => {}
            }
        }
    }
}
