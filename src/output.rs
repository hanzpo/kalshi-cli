use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use serde::Serialize;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

pub trait TableDisplay {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

pub fn print_json<T: Serialize + ?Sized>(data: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

pub fn print_table<T: TableDisplay>(items: &[T]) -> Result<()> {
    if items.is_empty() {
        println!("No results.");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(T::headers());
    for item in items {
        table.add_row(item.row());
    }
    println!("{table}");
    Ok(())
}

pub fn output<T: Serialize + TableDisplay>(data: &[T], format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => print_json(data),
        OutputFormat::Table => print_table(data),
    }
}

pub fn output_one<T: Serialize + TableDisplay + Clone>(
    data: &T,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => print_json(data),
        OutputFormat::Table => print_table(&[data.clone()]),
    }
}
