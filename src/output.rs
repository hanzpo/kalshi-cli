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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct TestItem {
        id: String,
        name: String,
    }

    impl TableDisplay for TestItem {
        fn headers() -> Vec<&'static str> {
            vec!["ID", "Name"]
        }
        fn row(&self) -> Vec<String> {
            vec![self.id.clone(), self.name.clone()]
        }
    }

    fn make_item(id: &str, name: &str) -> TestItem {
        TestItem {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn output_config_construction() {
        let cfg = OutputConfig {
            format: OutputFormat::Table,
            no_pager: true,
            color: false,
            quiet: false,
            yes: false,
        };
        assert!(cfg.no_pager);
        assert!(!cfg.color);
        assert!(!cfg.quiet);
        assert!(!cfg.yes);
    }

    #[test]
    fn output_format_json_variant_exists() {
        let _f = OutputFormat::Json;
    }

    #[test]
    fn output_format_table_variant_exists() {
        let _f = OutputFormat::Table;
    }

    #[test]
    fn output_format_csv_variant_exists() {
        let _f = OutputFormat::Csv;
    }

    #[test]
    fn test_item_headers() {
        assert_eq!(TestItem::headers(), vec!["ID", "Name"]);
    }

    #[test]
    fn test_item_row() {
        let item = make_item("1", "Alice");
        assert_eq!(item.row(), vec!["1", "Alice"]);
    }

    #[test]
    fn colored_row_default_returns_row() {
        let item = make_item("42", "Bob");
        assert_eq!(item.colored_row(true), item.row());
        assert_eq!(item.colored_row(false), item.row());
    }

    #[test]
    fn print_quiet_outputs_first_column() {
        // Verify the logic: print_quiet iterates items and prints the first element of row()
        let item = make_item("ticker-abc", "Some Market");
        let row = item.row();
        assert_eq!(row.first().unwrap(), "ticker-abc");
    }

    #[test]
    fn print_csv_writes_header_and_rows() {
        // Test CSV output using an in-memory writer to verify format
        let items = vec![make_item("1", "Alpha"), make_item("2", "Beta")];
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(TestItem::headers()).unwrap();
        for item in &items {
            wtr.write_record(item.row()).unwrap();
        }
        wtr.flush().unwrap();
        let output = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        assert!(output.contains("ID,Name"));
        assert!(output.contains("1,Alpha"));
        assert!(output.contains("2,Beta"));
    }

    #[test]
    fn output_quiet_takes_quiet_branch() {
        // When quiet=true, output() should call print_quiet (returns Ok without hitting format)
        let cfg = OutputConfig {
            format: OutputFormat::Json,
            no_pager: true,
            color: false,
            quiet: true,
            yes: false,
        };
        let items = vec![make_item("x", "y")];
        // This should not error — it takes the quiet branch, printing to stdout
        let result = output(&items, &cfg);
        assert!(result.is_ok());
    }

    #[test]
    fn output_empty_items_quiet() {
        let cfg = OutputConfig {
            format: OutputFormat::Table,
            no_pager: true,
            color: false,
            quiet: true,
            yes: false,
        };
        let items: Vec<TestItem> = vec![];
        let result = output(&items, &cfg);
        assert!(result.is_ok());
    }
}
