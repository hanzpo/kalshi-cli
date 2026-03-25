mod auth;
mod browse;
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
use output::OutputConfig;

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
    let out = OutputConfig {
        format: cli.output,
        no_pager: cli.no_pager,
    };

    match cli.command {
        Command::Config { .. } => unreachable!(),
        Command::Exchange { cmd } => {
            commands::exchange::execute(&client, cmd, &out).await?;
        }
        Command::Market { cmd } => {
            commands::markets::execute(&client, cmd, &out).await?;
        }
        Command::Event { cmd } => {
            commands::events::execute(&client, cmd, &out).await?;
        }
        Command::Series { cmd } => {
            commands::series::execute(&client, cmd, &out).await?;
        }
        Command::Order { cmd } => {
            commands::orders::execute(&client, cmd, &out).await?;
        }
        Command::OrderGroup { cmd } => {
            commands::order_groups::execute(&client, cmd, &out).await?;
        }
        Command::Portfolio { cmd } => {
            commands::portfolio::execute(&client, cmd, &out).await?;
        }
        Command::Historical { cmd } => {
            commands::historical::execute(&client, cmd, &out).await?;
        }
        Command::Subaccount { cmd } => {
            commands::subaccounts::execute(&client, cmd, &out).await?;
        }
        Command::ApiKey { cmd } => {
            commands::api_keys::execute(&client, cmd, &out).await?;
        }
        Command::Rfq { cmd } => {
            commands::communications::execute_rfq(&client, cmd, &out).await?;
        }
        Command::Quote { cmd } => {
            commands::communications::execute_quote(&client, cmd, &out).await?;
        }
    }

    Ok(())
}
