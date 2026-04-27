---
name: kalshi
description: Use this skill when the user wants to trade on Kalshi, check Kalshi prediction market prices, place/cancel orders, view their portfolio (balance, positions, fills, settlements), watch live market data, set price alerts, or otherwise interact with the Kalshi prediction market API. Triggers on mentions of "Kalshi", prediction markets, contract tickers, the `kalshi` CLI, `kalshi buy`/`kalshi sell`/`kalshi status`, market orderbooks, event series, or RFQs/quotes on Kalshi.
---

# Kalshi CLI

The `kalshi` binary is a full-featured CLI for the [Kalshi](https://kalshi.com) prediction market API. Contracts trade in **cents** (0–100), settle at **$1.00** if YES resolves true (or $0.00 if NO), and can be bought on either the YES or NO side.

Use this skill whenever the user wants to hit that API from the terminal.

## How to drive it

1. **Always check installation first** — run `kalshi --version`. If not installed, the repo root has `cargo install --path .`.
2. **Authentication is required for most commands.** `status`, `buy`, `sell`, `portfolio *`, `order *`, `watch *`, `export *`, `alert *`, `api-key *`, `account *`, `subaccount *`, and `exchange user-data-timestamp` all require auth. Public read commands (`market list`, `market get`, `market search`, `event *`, `series *`, `historical market`, `ping`, `exchange status`, `ticker`, `url`) do not.
3. **Run `kalshi config show` before first use** to confirm the user has a working config. If not, direct them to `kalshi config init` (interactive — the user must run it themselves, not you).
4. **For programmatic parsing, always pass `--output json`.** The default `table` format is designed for humans and includes color codes and headings.
5. **For scripting, also pass `--no-pager --no-color -y`** to avoid paging prompts, ANSI codes, and confirmation dialogs.
6. **Prices are integer cents** (e.g. `45` for $0.45). Quantities are integer number of contracts.
7. **Respect the 5 req/sec rate limit.** The CLI throttles automatically and retries on 429, but avoid tight loops.
8. **Use `--demo` for any non-production testing** or when the user explicitly wants paper trading.

## Quick decision tree

| User wants to... | Command |
|---|---|
| See account snapshot | `kalshi status` |
| Find a market | `kalshi market search "<query>"` or `kalshi market hot` |
| Look up a known ticker | `kalshi market get <TICKER>` |
| See the orderbook | `kalshi market orderbook <TICKER>` (requires auth) |
| Simulate a fill | `kalshi market analyze <TICKER> --buy <qty>` |
| Place an order | `kalshi buy <TICKER> <qty> --at <cents>` |
| Cancel everything | `kalshi cancel-all` |
| See positions | `kalshi portfolio position` |
| Watch live prices | `kalshi watch ticker <TICKER1> <TICKER2>` |
| Export data | `kalshi export fill -o fills.csv --format csv` |
| Get market URL from a ticker | `kalshi url <TICKER>` |
| Extract tickers from a Kalshi web URL | `kalshi ticker <url>` |

See [commands.md](commands.md) for the complete reference and [workflows.md](workflows.md) for common end-to-end patterns.

## Global flags (always available)

| Flag | Purpose |
|---|---|
| `--output table\|json\|csv` | Output format; **use `json` when you need to parse output** |
| `--demo` | Hit the demo environment instead of prod |
| `--profile <name>` | Use a named profile from `~/.kalshi/config.toml` |
| `--config <path>` | Alternate config file |
| `--no-pager` | Disable the pager (set this for scripted use) |
| `--no-color` | Strip ANSI color codes |
| `-q, --quiet` | Print only IDs/tickers, one per line — ideal for piping |
| `-y, --yes` | Skip all confirmation prompts |

## Output format tips

- `--output json` returns structured JSON you can pipe into `jq`. Prefer this in every automation context.
- `--output csv` is available for tabular commands only (lists, portfolio, historical).
- The default `table` format uses Unicode borders and colors. Don't pipe it into other tools.
- `-q` makes many list commands output just the IDs — useful for `xargs` loops.

## Critical safety rules

- **Never place real-money orders without explicit user confirmation** that includes ticker, side, quantity, and price. Even with `-y` available, confirm in chat first unless the user has already given a direct, unambiguous instruction for this specific order.
- **Prefer `--demo`** for any exploratory or educational work.
- **Never run `kalshi cancel-all` without confirmation** unless the user said "cancel everything".
- **Don't retry failed orders** without understanding why they failed — duplicate fills cost real money.
- **Treat `config init` as interactive** — don't try to script it. If the user has no config, tell them to run it themselves.

## Kalshi domain concepts (quick refresher)

- **Ticker**: an all-caps identifier like `KXMARMAD-26-DUKE`. Markets usually have `SERIES-EVENT-MARKET` structure.
- **Series** > **Event** > **Market**: a series (e.g. "March Madness") contains events (e.g. "2026 tournament"), each containing markets (e.g. "Duke wins").
- **YES / NO side**: every contract has both a YES and a NO side. YES price + NO price ≈ 100 cents (the spread is the market's bid-ask).
- **Buy YES at 45¢**: costs 45¢, pays $1 if YES resolves, $0 otherwise. **Buy NO at 55¢**: costs 55¢, pays $1 if NO resolves.
- **Combo / multivariate markets**: `kalshi collection *` commands. Excluded from `market list` by default; pass `--include-combos` to see them.
- **RFQ / Quote**: block-trade mechanism for large orders. `kalshi rfq create` + `kalshi quote *`.
- **Order types**: `gtc` (good-till-cancel, default), `fok` (fill-or-kill), `ioc` (immediate-or-cancel). Plus `--post-only` and `--reduce-only` modifiers.

## When something goes wrong

| Symptom | Likely cause | Fix |
|---|---|---|
| `Authentication required` | No config / bad keys | `kalshi config show`, then `kalshi config init` if empty |
| `HTTP 429` spam | Rate-limited | CLI auto-retries; if you're looping, back off |
| `Invalid ticker` | Typo or market closed | `kalshi market search` to find the right ticker |
| Orderbook/fill commands return nothing | Market is closed/settled | `kalshi market get <TICKER>` to check status |
| `config init` hangs | It's interactive — user must run it | Don't invoke it yourself; ask the user to run it |
| Output gets paged/colored in scripts | Pager + TTY detection | Add `--no-pager --no-color` |
