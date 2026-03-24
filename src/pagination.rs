use std::future::Future;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PaginationOpts {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub all: bool,
}

pub async fn auto_paginate<T, F, Fut>(
    opts: &PaginationOpts,
    page_size: u32,
    fetcher: F,
) -> Result<Vec<T>>
where
    F: Fn(u32, Option<String>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>>,
{
    let mut all_items = Vec::new();
    let mut cursor = opts.cursor.clone();
    let target = if opts.all {
        u32::MAX
    } else {
        opts.limit.unwrap_or(page_size)
    };

    loop {
        let remaining = target.saturating_sub(all_items.len() as u32);
        let fetch_limit = remaining.min(page_size);
        if fetch_limit == 0 {
            break;
        }

        let (items, next_cursor) = fetcher(fetch_limit, cursor).await?;
        let done = items.is_empty()
            || next_cursor.as_ref().map_or(true, |c| c.is_empty());
        all_items.extend(items);

        if done || all_items.len() as u32 >= target {
            break;
        }
        cursor = next_cursor;
    }

    // Trim to exact limit if needed
    if !opts.all {
        if let Some(limit) = opts.limit {
            all_items.truncate(limit as usize);
        }
    }

    Ok(all_items)
}
