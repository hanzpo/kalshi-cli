use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use serde::Serialize;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}

/// Bundles output settings passed through commands.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub no_pager: bool,
    pub color: bool,
    pub quiet: bool,
    pub yes: bool,
}

pub trait TableDisplay {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
    fn colored_row(&self, _color: bool) -> Vec<String> {
        self.row()
    }
}

/// Write output through the system pager if stdout is a TTY and content is tall.
/// Falls back to direct stdout if pager is unavailable or --no-pager is set.
pub fn paged_print(content: &str, no_pager: bool) {
    let stdout = io::stdout();
    let is_tty = stdout.is_terminal();
    let term_height = terminal_height();

    let line_count = content.lines().count();
    let should_page = is_tty && !no_pager && line_count > term_height;

    if should_page {
        let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        // -R preserves ANSI colors, -F quits if content fits one screen, -X no alt screen
        let args = if pager.contains("less") {
            vec!["-RFX".to_string()]
        } else {
            vec![]
        };

        if let Ok(mut child) = Command::new(&pager)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(content.as_bytes());
            }
            let _ = child.wait();
            return;
        }
    }

    // Fallback: print directly
    print!("{content}");
}

fn terminal_height() -> usize {
    terminal_size::terminal_size()
        .map(|(_, h)| h.0 as usize)
        .unwrap_or(24)
}

pub fn print_json<T: Serialize + ?Sized>(data: &T, no_pager: bool) -> Result<()> {
    let text = serde_json::to_string_pretty(data)?;
    paged_print(&format!("{text}\n"), no_pager);
    Ok(())
}

pub fn print_table<T: TableDisplay>(items: &[T], no_pager: bool, color: bool) -> Result<()> {
    if items.is_empty() {
        println!("No results.");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(T::headers());
    for item in items {
        table.add_row(item.colored_row(color));
    }
    paged_print(&format!("{table}\n"), no_pager);
    Ok(())
}

pub fn print_csv<T: TableDisplay>(items: &[T]) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(T::headers())?;
    for item in items {
        wtr.write_record(item.row())?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn print_quiet<T: TableDisplay>(items: &[T]) {
    for item in items {
        let row = item.row();
        if let Some(first) = row.first() {
            println!("{}", first);
        }
    }
}

pub fn output<T: Serialize + TableDisplay>(data: &[T], cfg: &OutputConfig) -> Result<()> {
    if cfg.quiet {
        print_quiet(data);
        return Ok(());
    }
    match cfg.format {
        OutputFormat::Json => print_json(data, cfg.no_pager),
        OutputFormat::Table => print_table(data, cfg.no_pager, cfg.color),
        OutputFormat::Csv => print_csv(data),
    }
}

/// Output with a "showing X of more" hint when truncated
pub fn output_paginated<T: Serialize + TableDisplay>(
    data: &[T],
    has_more: bool,
    cfg: &OutputConfig,
) -> Result<()> {
    output(data, cfg)?;
    if has_more {
        eprintln!(
            "Showing {} results. Use --limit N for more, or --all for everything.",
            data.len()
        );
    }
    Ok(())
}

pub fn output_one<T: Serialize + TableDisplay + Clone>(
    data: &T,
    cfg: &OutputConfig,
) -> Result<()> {
    if cfg.quiet {
        print_quiet(&[data.clone()]);
        return Ok(());
    }
    match cfg.format {
        OutputFormat::Json => print_json(data, cfg.no_pager),
        OutputFormat::Table => print_table(&[data.clone()], cfg.no_pager, cfg.color),
        OutputFormat::Csv => print_csv(&[data.clone()]),
    }
}
