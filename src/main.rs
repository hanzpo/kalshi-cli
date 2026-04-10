mod alerts;
mod auth;
mod browse;
mod cli;
mod client;
mod color;
mod commands;
mod config;
mod confirm;
mod error;
mod models;
mod output;
mod pagination;
mod websocket;

use std::io::IsTerminal;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::{BANNER, Cli, Command};
use client::KalshiClient;
use config::Config;
use output::OutputConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 1 {
        print_banner(std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err());
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    let cli = Cli::parse_from(args);

    // Commands that don't need a client
    if matches!(cli.command, Command::Config { .. })
        && let Command::Config { cmd } = cli.command
    {
        return commands::config_cmd::execute(cmd).await;
    }
    if let Command::Completions { shell } = cli.command {
        commands::completions::execute(shell);
        return Ok(());
    }
    if let Command::Ping = cli.command {
        let demo = cli.demo;
        return commands::ping::execute(demo).await;
    }
    if let Command::Upgrade { check } = cli.command {
        return commands::upgrade::execute(check).await;
    }

    let config = Config::load(cli.config.as_deref())?;
    // If --demo is passed without --profile, auto-select the "demo" profile if it exists
    let profile = cli.profile.as_deref().or_else(|| {
        if cli.demo && config.profiles.contains_key("demo") {
            Some("demo")
        } else {
            None
        }
    });
    let config = config.resolve(profile)?;
    let demo = cli.demo || config.demo.unwrap_or(false);
    let client = KalshiClient::new(&config, demo)?;
    let color =
        !cli.no_color && std::env::var("NO_COLOR").is_err() && std::io::stdout().is_terminal();
    let out = OutputConfig {
        format: cli.output,
        no_pager: cli.no_pager,
        color,
        quiet: cli.quiet,
        yes: cli.yes,
    };

    dispatch(cli.command, &client, &config, demo, &out).await
}

pub fn print_banner(color: bool) {
    print_banner_to(color, false);
}

pub fn eprint_banner(color: bool) {
    print_banner_to(color, true);
}

fn print_banner_to(color: bool, stderr: bool) {
    if color {
        if stderr {
            eprint!("\x1b[32m{BANNER}\x1b[0m");
        } else {
            print!("\x1b[32m{BANNER}\x1b[0m");
        }
    } else if stderr {
        eprint!("{BANNER}");
    } else {
        print!("{BANNER}");
    }
}

pub async fn dispatch(
    command: Command,
    client: &KalshiClient,
    config: &Config,
    demo: bool,
    out: &OutputConfig,
) -> Result<()> {
    match command {
        Command::Config { .. } | Command::Completions { .. } => unreachable!(),
        Command::Exchange { cmd } => {
            commands::exchange::execute(client, cmd, out).await?;
        }
        Command::Market { cmd } => {
            commands::markets::execute(client, cmd, out).await?;
        }
        Command::Event { cmd } => {
            commands::events::execute(client, cmd, out).await?;
        }
        Command::Series { cmd } => {
            commands::series::execute(client, cmd, out).await?;
        }
        Command::Order { cmd } => {
            commands::orders::execute(client, cmd, out).await?;
        }
        Command::OrderGroup { cmd } => {
            commands::order_groups::execute(client, cmd, out).await?;
        }
        Command::Portfolio { cmd } => {
            commands::portfolio::execute(client, cmd, out).await?;
        }
        Command::Historical { cmd } => {
            commands::historical::execute(client, cmd, out).await?;
        }
        Command::Subaccount { cmd } => {
            commands::subaccounts::execute(client, cmd, out).await?;
        }
        Command::ApiKey { cmd } => {
            commands::api_keys::execute(client, cmd, out).await?;
        }
        Command::Rfq { cmd } => {
            commands::communications::execute_rfq(client, cmd, out).await?;
        }
        Command::Quote { cmd } => {
            commands::communications::execute_quote(client, cmd, out).await?;
        }
        Command::Account { cmd } => {
            commands::account::execute(client, cmd, out).await?;
        }
        Command::Search { cmd } => {
            commands::search::execute(client, cmd, out).await?;
        }
        Command::Milestone { cmd } => {
            commands::milestone::execute(client, cmd, out).await?;
        }
        Command::LiveData { cmd } => {
            commands::live_data::execute(client, cmd, out).await?;
        }
        Command::StructuredTarget { cmd } => {
            commands::structured_target::execute(client, cmd, out).await?;
        }
        Command::IncentiveProgram { cmd } => {
            commands::incentive_program::execute(client, cmd, out).await?;
        }
        Command::Fcm { cmd } => {
            commands::fcm::execute(client, cmd, out).await?;
        }
        Command::Collection { cmd } => {
            commands::collection::execute(client, cmd, out).await?;
        }
        Command::Status => {
            commands::status::execute(client, out).await?;
        }
        Command::Buy {
            ticker,
            quantity,
            yes,
            no,
            at,
        } => {
            commands::trade::execute_buy(client, &ticker, quantity, yes, no, at, out).await?;
        }
        Command::Sell {
            ticker,
            quantity,
            yes,
            no,
            at,
        } => {
            commands::trade::execute_sell(client, &ticker, quantity, yes, no, at, out).await?;
        }
        Command::Close { ticker } => {
            commands::trade::execute_close(client, &ticker, out).await?;
        }
        Command::CancelAll { ticker } => {
            commands::trade::execute_cancel_all(client, ticker.as_deref(), out).await?;
        }
        Command::Export { cmd } => {
            commands::export::execute(client, cmd, out).await?;
        }
        Command::Watch { cmd } => {
            let ws = websocket::KalshiWebSocket::new(config, demo)?;
            commands::watch::execute(&ws, cmd, out).await?;
        }
        Command::Alert { cmd } => {
            let ws = websocket::KalshiWebSocket::new(config, demo)?;
            commands::alert::execute(cmd, &ws, out).await?;
        }
        Command::Ticker { url } => {
            commands::ticker::execute(client, &url).await?;
        }
        Command::Url { ticker, open } => {
            commands::url::execute(client, &ticker, open).await?;
        }
        Command::Ping => {
            commands::ping::execute(demo).await?;
        }
        Command::Shell => {
            commands::shell::execute(client, config, demo, out).await?;
        }
        Command::Upgrade { check } => {
            commands::upgrade::execute(check).await?;
        }
    }

    Ok(())
}
