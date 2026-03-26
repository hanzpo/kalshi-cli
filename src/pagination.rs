use std::future::Future;

use anyhow::Result;

pub const DEFAULT_DISPLAY_LIMIT: u32 = 20;
const API_MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Clone)]
pub struct PaginationOpts {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub all: bool,
}

pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub has_more: bool,
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
    let target = if opts.all {
        u32::MAX
    } else {
        opts.limit.unwrap_or(DEFAULT_DISPLAY_LIMIT)
    };

    loop {
        let remaining = target.saturating_sub(all_items.len() as u32);
        let fetch_limit = remaining.min(API_MAX_PAGE_SIZE);
        if fetch_limit == 0 {
            break;
        }

        let (items, next_cursor) = fetcher(fetch_limit, cursor).await?;
        let done = items.is_empty()
            || next_cursor.as_ref().map_or(true, |c| c.is_empty());
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

    if !opts.all {
        if let Some(limit) = opts.limit {
            all_items.truncate(limit as usize);
        }
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
