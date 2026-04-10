use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::cli::{Cli, Command};
use crate::client::KalshiClient;
use crate::config::Config;
use crate::dispatch;
use crate::eprint_banner;
use crate::output::OutputConfig;

pub async fn execute(
    client: &KalshiClient,
    config: &Config,
    demo: bool,
    out: &OutputConfig,
) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let history_path = Config::config_dir().join("history.txt");
    let _ = rl.load_history(&history_path);

    eprint_banner(out.color);
    eprintln!("Kalshi interactive shell. Type 'help' for commands, 'exit' to quit.");

    loop {
        match rl.readline("kalshi> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if matches!(line, "exit" | "quit") {
                    break;
                }
                let _ = rl.add_history_entry(line);

                let words = match shell_words::split(line) {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Parse error: {e}");
                        continue;
                    }
                };

                // Prepend "kalshi" so clap can parse as if invoked from the command line
                let mut args = vec!["kalshi".to_string()];
                args.extend(words);

                match Cli::try_parse_from(&args) {
                    Ok(cli) => {
                        // Prevent shell-in-shell
                        if matches!(cli.command, Command::Shell) {
                            eprintln!("Already in shell mode.");
                            continue;
                        }
                        // Config and completions don't make sense in the shell
                        if matches!(
                            cli.command,
                            Command::Config { .. } | Command::Completions { .. }
                        ) {
                            eprintln!("This command is not available in shell mode.");
                            continue;
                        }
                        if let Err(e) =
                            Box::pin(dispatch(cli.command, client, config, demo, out)).await
                        {
                            eprintln!("Error: {e}");
                        }
                    }
                    Err(e) => {
                        // clap prints help/error messages
                        let _ = e.print();
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}
