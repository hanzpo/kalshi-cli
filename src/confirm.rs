use anyhow::{Result, bail};
use std::io::{self, IsTerminal, Write};

pub fn confirm(message: &str, skip: bool) -> Result<bool> {
    if skip {
        return Ok(true);
    }
    if !io::stdout().is_terminal() {
        bail!("Cannot confirm in non-interactive mode. Use --yes to skip.");
    }
    eprint!("{} [y/N] ", message);
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
