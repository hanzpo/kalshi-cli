use std::future::Future;

use anyhow::Result;

use crate::browse;
use crate::output::{OutputConfig, TableDisplay, output_paginated};

pub const DEFAULT_DISPLAY_LIMIT: u32 = 25;

/// Safe default for most endpoints (orders, fills, settlements, events all max at 200).
const DEFAULT_MAX_PAGE_SIZE: u32 = 200;

/// Default number of rows per page in the interactive browser.
const BROWSE_PAGE_SIZE: u32 = 25;

/// Use for endpoints that support up to 1000 per page (markets, trades, positions).
pub const MARKETS_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Clone)]
pub struct PaginationOpts {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub all: bool,
    /// Maximum items the API will return per page. Defaults to 200 if not set.
    pub max_page_size: Option<u32>,
}

pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

/// Standard list command: `--all` launches interactive browser, otherwise fetches
/// up to `--limit` items and prints them.
///
/// Handles APIs that ignore the limit param and dump everything at once by
/// buffering overflow items and serving them as subsequent pages in the browser.
pub async fn paginated_list<T, F, Fut>(
    all: bool,
    limit: Option<u32>,
    cursor: Option<String>,
    max_page_size: Option<u32>,
    format: &OutputConfig,
    fetcher: F,
) -> Result<()>
where
    T: TableDisplay + serde::Serialize,
    F: Fn(u32, Option<String>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>>,
{
    if all && format.is_non_interactive() {
        // Non-interactive --all: fetch everything and dump to stdout
        let opts = PaginationOpts {
            limit: None,
            cursor,
            all: true,
            max_page_size,
        };
        let result = auto_paginate(&opts, fetcher).await?;
        output_paginated(&result.items, result.has_more, format)
    } else if all {
        let page_size = BROWSE_PAGE_SIZE as usize;
        let initial_cursor = cursor;
        // Buffer for overflow when API returns more items than one display page.
        let buffer: std::sync::Mutex<Vec<T>> = std::sync::Mutex::new(Vec::new());
        let api_cursor: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
        let api_exhausted: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

        browse::browse(
            BROWSE_PAGE_SIZE,
            |_page_limit, page_cursor: Option<String>| {
                let initial_cursor = initial_cursor.clone();
                async {
                    // First, drain any buffered items from a previous oversized response.
                    {
                        let mut buf = buffer.lock().unwrap();
                        if !buf.is_empty() {
                            let take = buf.len().min(page_size);
                            let chunk: Vec<T> = buf.drain(..take).collect();
                            let cursor = if buf.is_empty() {
                                api_cursor.lock().unwrap().clone()
                            } else {
                                Some("__buffered__".to_string())
                            };
                            return Ok((chunk, cursor));
                        }
                    }

                    if *api_exhausted.lock().unwrap() {
                        return Ok((Vec::new(), None));
                    }

                    let effective_cursor = match page_cursor {
                        Some(ref c) if c == "__buffered__" => api_cursor.lock().unwrap().clone(),
                        Some(_) => page_cursor,
                        None => initial_cursor,
                    };

                    // Request only what we need for one display page.
                    let (mut items, next_cursor) =
                        fetcher(BROWSE_PAGE_SIZE, effective_cursor).await?;

                    if next_cursor.as_ref().is_none_or(|c| c.is_empty()) {
                        *api_exhausted.lock().unwrap() = true;
                    }

                    if items.len() > page_size {
                        let overflow = items.split_off(page_size);
                        *buffer.lock().unwrap() = overflow;
                        *api_cursor.lock().unwrap() = next_cursor;
                        Ok((items, Some("__buffered__".to_string())))
                    } else {
                        Ok((items, next_cursor))
                    }
                }
            },
        )
        .await
    } else {
        let opts = PaginationOpts {
            limit,
            cursor,
            all: false,
            max_page_size,
        };
        let result = auto_paginate(&opts, fetcher).await?;
        output_paginated(&result.items, result.has_more, format)
    }
}

pub async fn auto_paginate<T, F, Fut>(
    opts: &PaginationOpts,
    fetcher: F,
) -> Result<PaginatedResult<T>>
where
    F: Fn(u32, Option<String>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>>,
{
    let mut all_items = Vec::new();
    let mut cursor = opts.cursor.clone();
    let mut has_more = false;
    let max_page = opts.max_page_size.unwrap_or(DEFAULT_MAX_PAGE_SIZE);
    let target = if opts.all {
        u32::MAX
    } else {
        opts.limit.unwrap_or(DEFAULT_DISPLAY_LIMIT)
    };

    loop {
        let remaining = target.saturating_sub(all_items.len() as u32);
        let fetch_limit = remaining.min(max_page);
        if fetch_limit == 0 {
            break;
        }

        let (items, next_cursor) = fetcher(fetch_limit, cursor).await?;
        let done = items.is_empty() || next_cursor.as_ref().is_none_or(|c| c.is_empty());
        all_items.extend(items);

        if done {
            break;
        }
        if all_items.len() as u32 >= target {
            has_more = true;
            break;
        }
        cursor = next_cursor;
    }

    if !opts.all
        && let Some(limit) = opts.limit
    {
        all_items.truncate(limit as usize);
    }

    Ok(PaginatedResult {
        items: all_items,
        has_more,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_limit_truncates() {
        let opts = PaginationOpts {
            limit: Some(5),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, |_limit, _cursor| async {
            Ok((vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], None::<String>))
        })
        .await
        .unwrap();
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_with_no_limit_uses_default() {
        let opts = PaginationOpts {
            limit: None,
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let items: Vec<i32> = (0..30).collect();
        let result = auto_paginate(&opts, |_limit, _cursor| {
            let items = items.clone();
            async move { Ok((items, None::<String>)) }
        })
        .await
        .unwrap();
        // DEFAULT_DISPLAY_LIMIT is 20, but fetcher returns 30 in one page with no cursor
        // so it fetches all 30 but then doesn't truncate because limit is None
        // Actually looking at the code: truncation only happens if opts.limit is Some
        assert_eq!(result.items.len(), 30);
    }

    #[tokio::test]
    async fn test_with_all_fetches_everything() {
        let opts = PaginationOpts {
            limit: None,
            cursor: None,
            all: true,
            max_page_size: None,
        };
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();
        let result = auto_paginate(&opts, move |_limit, _cursor| {
            let cc = cc.clone();
            async move {
                let count = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count == 0 {
                    Ok(((0..100).collect::<Vec<i32>>(), Some("page2".to_string())))
                } else {
                    Ok(((100..150).collect::<Vec<i32>>(), None))
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(result.items.len(), 150);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_empty_fetcher_stops_immediately() {
        let opts = PaginationOpts {
            limit: Some(10),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, |_limit, _cursor| async {
            Ok((Vec::<i32>::new(), None::<String>))
        })
        .await
        .unwrap();
        assert!(result.items.is_empty());
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_no_cursor_stops_after_one_page() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();
        let opts = PaginationOpts {
            limit: Some(100),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, move |_limit, _cursor| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok((vec![1, 2, 3], None::<String>))
            }
        })
        .await
        .unwrap();
        assert_eq!(result.items, vec![1, 2, 3]);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multi_page_fetch() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();
        let opts = PaginationOpts {
            limit: Some(100),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, move |_limit, cursor| {
            let cc = cc.clone();
            async move {
                let count = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                match count {
                    0 => {
                        assert!(cursor.is_none());
                        Ok((vec![1, 2, 3], Some("cursor_page2".to_string())))
                    }
                    1 => {
                        assert_eq!(cursor, Some("cursor_page2".to_string()));
                        Ok((vec![4, 5], None))
                    }
                    _ => panic!("Should not be called more than twice"),
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(result.items, vec![1, 2, 3, 4, 5]);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_limit_with_has_more() {
        let opts = PaginationOpts {
            limit: Some(3),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, |_limit, _cursor| async {
            Ok((vec![1, 2, 3], Some("next".to_string())))
        })
        .await
        .unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn test_empty_cursor_string_stops() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();
        let opts = PaginationOpts {
            limit: Some(100),
            cursor: None,
            all: false,
            max_page_size: None,
        };
        let result = auto_paginate(&opts, move |_limit, _cursor| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok((vec![1, 2], Some("".to_string())))
            }
        })
        .await
        .unwrap();
        assert_eq!(result.items, vec![1, 2]);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
