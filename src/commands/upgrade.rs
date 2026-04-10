use anyhow::Result;
use serde::Deserialize;

const GITHUB_REPO: &str = "hanzpo/kalshi-cli";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

pub async fn execute(check: bool) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    eprintln!("Current version: v{current}");

    let client = reqwest::Client::builder()
        .user_agent("kalshi-cli")
        .build()?;

    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        eprintln!("No releases found. Automatic upgrades will be available once releases are published.");
        return Ok(());
    }

    if !resp.status().is_success() {
        anyhow::bail!("Failed to check for updates: HTTP {}", resp.status());
    }

    let release: Release = resp.json().await?;
    let latest = release.tag_name.trim_start_matches('v');

    if latest == current {
        eprintln!("Already up to date.");
    } else {
        eprintln!("New version available: v{latest}");
        eprintln!("Release: {}", release.html_url);
        if !check {
            eprintln!("\nAutomatic download is not yet available.");
            eprintln!("To upgrade, visit the release page or run:");
            eprintln!("  cargo install --git https://github.com/{GITHUB_REPO}");
        }
    }

    Ok(())
}
