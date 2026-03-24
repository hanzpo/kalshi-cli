mod auth;
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod models;
mod output;
mod pagination;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use client::KalshiClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Config command doesn't need a client
    if matches!(cli.command, Command::Config { .. }) {
        if let Command::Config { cmd } = cli.command {
            return commands::config_cmd::execute(cmd).await;
        }
    }

    let config = Config::load(cli.config.as_deref())?;
    let demo = cli.demo || config.demo.unwrap_or(false);
    let client = KalshiClient::new(&config, demo)?;

    match cli.command {
        Command::Config { .. } => unreachable!(),
        Command::Exchange { cmd } => {
            commands::exchange::execute(&client, cmd, &cli.output).await?;
        }
        Command::Market { cmd } => {
            commands::markets::execute(&client, cmd, &cli.output).await?;
        }
        Command::Event { cmd } => {
            commands::events::execute(&client, cmd, &cli.output).await?;
        }
        Command::Series { cmd } => {
            commands::series::execute(&client, cmd, &cli.output).await?;
        }
        Command::Order { cmd } => {
            commands::orders::execute(&client, cmd, &cli.output).await?;
        }
        Command::OrderGroup { cmd } => {
            commands::order_groups::execute(&client, cmd, &cli.output).await?;
        }
        Command::Portfolio { cmd } => {
            commands::portfolio::execute(&client, cmd, &cli.output).await?;
        }
        Command::Historical { cmd } => {
            commands::historical::execute(&client, cmd, &cli.output).await?;
        }
        Command::Subaccount { cmd } => {
            commands::subaccounts::execute(&client, cmd, &cli.output).await?;
        }
        Command::ApiKey { cmd } => {
            commands::api_keys::execute(&client, cmd, &cli.output).await?;
        }
        Command::Rfq { cmd } => {
            commands::communications::execute_rfq(&client, cmd, &cli.output).await?;
        }
        Command::Quote { cmd } => {
            commands::communications::execute_quote(&client, cmd, &cli.output).await?;
        }
    }

    Ok(())
}
