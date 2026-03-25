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
