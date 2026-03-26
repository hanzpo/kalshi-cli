use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::Cli;

pub fn execute(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "kalshi", &mut std::io::stdout());
}
