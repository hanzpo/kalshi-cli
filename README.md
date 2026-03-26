# Kalshi CLI

A fast, full-featured command-line interface for the [Kalshi](https://kalshi.com) prediction market API. Trade contracts, monitor markets in real time, manage your portfolio, and export data -- all from your terminal.

```
 ██╗  ██╗ █████╗ ██╗     ███████╗██╗  ██╗██╗
 ██║ ██╔╝██╔══██╗██║     ██╔════╝██║  ██║██║
 █████╔╝ ███████║██║     ███████╗███████║██║
 ██╔═██╗ ██╔══██║██║     ╚════██║██╔══██║██║
 ██║  ██╗██║  ██║███████╗███████║██║  ██║██║
 ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝╚═╝
              Trade on anything.
```

## Features

- **Full API coverage** -- markets, events, series, orders, portfolio, historical data, RFQs, collections, and more
- **Quick trading shortcuts** -- `kalshi buy`, `kalshi sell`, `kalshi close` for fast order execution
- **Real-time WebSocket feeds** -- live prices, trades, orderbook deltas, fills, and position updates
- **Price alerts** -- set threshold alerts with native desktop notifications (macOS and Linux)
- **Market analytics** -- hot markets, expiring soon, bid-ask spreads, orderbook fill simulation
- **Semantic search** -- find markets using natural language queries
- **Data export** -- export fills, positions, and settlements to CSV, JSON, or JSONL
- **Interactive pagination** -- browse large result sets page-by-page with keyboard navigation
- **Multiple output formats** -- table (with color), JSON, and CSV output
- **Profile support** -- manage multiple accounts/environments via named config profiles
- **Demo mode** -- test against Kalshi's demo environment with `--demo`
- **Shell completions** -- auto-completions for bash, zsh, fish, and more

## Installation

### From source

Requires [Rust](https://rustup.rs/) (edition 2024).

```bash
git clone https://github.com/your-username/kalshi-cli.git
cd kalshi-cli
cargo install --path .
```

The binary `kalshi` will be installed to `~/.cargo/bin/`.

### Build without installing

```bash
cargo build --release
# Binary at target/release/kalshi
```

## Quick start

### 1. Get an API key

Log in to [Kalshi](https://kalshi.com) and generate an API key from your account settings. You'll need the **API key ID** and the **private key** (PEM format).

### 2. Configure the CLI

```bash
kalshi config init
```

This interactive wizard creates `~/.kalshi/config.toml` with your API credentials. You can also set credentials via environment variables:

```bash
export KALSHI_API_KEY_ID="your-key-id"
export KALSHI_PRIVATE_KEY_PATH="/path/to/private.pem"
# or inline:
export KALSHI_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n..."
```

### 3. Check your status

```bash
kalshi status
```

This shows your balance, open positions, resting orders, and recent fills in a single dashboard view.

### 4. Start trading

```bash
# Browse hot markets
kalshi market hot

# Search for markets
kalshi market search "will it rain in NYC"

# Buy 10 Yes contracts at 45 cents
kalshi buy RAIN-NYC-24 10 --at 45

# Check your positions
kalshi portfolio position

# Watch live prices
kalshi watch ticker RAIN-NYC-24
```

## Usage

### Quick actions

```bash
kalshi status                       # Dashboard: balance, positions, orders, fills
kalshi buy <ticker> <qty> [--at N]  # Buy contracts (--yes/--no for side)
kalshi sell <ticker> <qty> [--at N] # Sell contracts
kalshi close <ticker>               # Close an open position
kalshi cancel-all [--ticker T]      # Cancel all resting orders
```

### Market data

```bash
kalshi market list --status open    # List open markets
kalshi market get TICKER            # Get market details
kalshi market search "query"        # Semantic search
kalshi market orderbook TICKER      # View orderbook depth
kalshi market hot --limit 10        # Top markets by volume
kalshi market expiring --within 6   # Expiring within 6 hours
kalshi market spread                # Widest bid-ask spreads
kalshi market analyze TICKER --buy 100  # Simulate filling 100 contracts
```

### Order management

```bash
kalshi order create TICKER --side yes --action buy --quantity 10 --yes-price 45
kalshi order list --status resting
kalshi order cancel ORDER_ID
kalshi order amend ORDER_ID --ticker T --side yes --action buy --quantity 20
kalshi order batch-create --file orders.json   # Up to 20 orders
kalshi order batch-cancel --ticker TICKER
```

### Portfolio

```bash
kalshi portfolio balance            # Account balance
kalshi portfolio position --all     # Browse all positions
kalshi portfolio fill --ticker T    # Trade fills
kalshi portfolio settlement         # Settlement history
kalshi portfolio summary            # Portfolio analytics
```

### Real-time feeds (WebSocket)

```bash
kalshi watch ticker TICKER1 TICKER2   # Live prices
kalshi watch trade TICKER             # Live trades
kalshi watch orderbook TICKER         # Orderbook deltas
kalshi watch fill                     # Your fill notifications
kalshi watch position                 # Position updates
kalshi watch order                    # Order status updates
kalshi watch lifecycle                # Market lifecycle events
```

### Price alerts

```bash
kalshi alert add TICKER --above 75 --below 25
kalshi alert list
kalshi alert remove ALERT_ID
kalshi alert watch                    # Monitor alerts via WebSocket
```

### Data export

```bash
kalshi export fill -o fills.csv --format csv --since 1700000000
kalshi export position -o positions.json --format json
kalshi export settlement -o settlements.jsonl --format jsonl
```

### Historical data

```bash
kalshi historical market --ticker T
kalshi historical trade --all
kalshi historical candlestick TICKER --series-ticker S
kalshi historical fill               # Your past fills
kalshi historical order              # Your past orders
```

See [COMMANDS.md](COMMANDS.md) for the full command reference.

## Configuration

The config file lives at `~/.kalshi/config.toml`:

```toml
api_key_id = "your-key-id"
private_key_path = "/path/to/private.pem"
demo = false
default_output = "table"
```

### Profiles

Manage multiple accounts or switch between production and demo:

```bash
kalshi config profile-add trading
kalshi config profile-list
kalshi --profile trading market list
```

```toml
[profiles.trading]
api_key_id = "other-key-id"
private_key_path = "/path/to/other.pem"

[profiles.demo]
api_key_id = "demo-key-id"
private_key_path = "/path/to/demo.pem"
demo = true
```

### Environment variables

| Variable | Description |
|---|---|
| `KALSHI_API_KEY_ID` | API key ID (overrides config) |
| `KALSHI_PRIVATE_KEY_PATH` | Path to private key PEM file |
| `KALSHI_PRIVATE_KEY` | Private key contents (inline) |
| `KALSHI_PROFILE` | Default profile name |

## Global flags

| Flag | Description |
|---|---|
| `--demo` | Use demo environment |
| `--config <path>` | Config file path |
| `--output table\|json\|csv` | Output format (default: table) |
| `--profile <name>` | Config profile to use |
| `--no-pager` | Disable automatic paging |
| `--no-color` | Disable colored output |
| `-q, --quiet` | Print only IDs/tickers (one per line) |
| `-y, --yes` | Skip confirmation prompts |
| `-v, --verbose` | Show HTTP request details |

## Shell completions

Generate completions for your shell:

```bash
# Bash
kalshi completions bash > ~/.local/share/bash-completion/completions/kalshi

# Zsh
kalshi completions zsh > ~/.zfunc/_kalshi

# Fish
kalshi completions fish > ~/.config/fish/completions/kalshi.fish
```

## Authentication

The CLI authenticates using RSA-PSS signatures (SHA-256). Each request is signed with your private key -- no passwords or tokens are stored or transmitted. The signing process:

1. Constructs a message: `{timestamp_ms}{HTTP_METHOD}{path}`
2. Signs it with RSA-PSS using SHA-256
3. Sends the signature, key ID, and timestamp as HTTP headers

This is the same authentication method used by Kalshi's official API.

## Rate limits

The CLI automatically throttles requests to 5/second and retries with exponential backoff on rate limit responses (HTTP 429). The interactive browser shows a countdown timer during rate limit waits.

## Demo environment

Test without risking real money:

```bash
# Per-command
kalshi --demo market list

# Or set in config
kalshi config init  # Choose demo during setup
```
