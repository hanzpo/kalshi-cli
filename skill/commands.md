# Kalshi CLI — Complete Command Reference

Every command supports the global flags documented in `SKILL.md` (`--output`, `--demo`, `--profile`, `--no-pager`, `--no-color`, `-q`, `-y`).

Commands marked **(auth)** require a configured API key.

---

## Quick actions

### `kalshi status` **(auth)**
Dashboard snapshot: balance, open positions, resting orders, recent fills. Ideal as a "what's going on" check.

### `kalshi buy <ticker> <quantity>` **(auth)**
Fast-path buy. Defaults to YES side.
- `--yes` / `--no` — pick the side (default YES)
- `--at <cents>` — limit price in cents; without it, order is market-like at the current ask

### `kalshi sell <ticker> <quantity>` **(auth)**
Fast-path sell. Same flags as `buy`.

### `kalshi close <ticker>` **(auth)**
Close an open position in `<ticker>` at the current best price.

### `kalshi cancel-all` **(auth)**
Cancel every resting order. Filter with `--ticker <TICKER>` to scope.

---

## Market data

### `kalshi market list`
List markets (paginated).
- `--status open|closed|settled`
- `--series-ticker <S>` — filter by series
- `--event-ticker <E>` — filter by event
- `--include-combos` — include combo/multivariate markets (excluded by default)
- `--search <keyword>` — case-insensitive keyword match on title/ticker
- `--all` — interactive browser, page by page
- `--limit <N>` / `--cursor <C>` — manual pagination

### `kalshi market get <ticker>`
Single market details: title, prices, volume, open interest, close/settle times.

### `kalshi market search <query>`
Server-side semantic/fuzzy search (uses Kalshi's internal `/v1/search/series` endpoint). Returns series with nested events and markets.
- `--limit <N>` / `--cursor <C>` / `--all`

### `kalshi market trade` (alias `trades`)
Recent trades.
- `<ticker>` positional (optional — omit for all trades)
- `--min-ts` / `--max-ts` — unix timestamp range
- `--limit` / `--cursor` / `--all`

### `kalshi market candlestick <ticker>` (alias `candlesticks`)
OHLCV data.
- `--series-ticker <S>` (required)
- `--period <minutes>` — e.g. `1`, `60`, `1440`
- `--start-ts` / `--end-ts`

### `kalshi market candlestick-batch`
Candlesticks for up to 100 markets at once.
- `--tickers <T1,T2,...>` — comma-separated

### `kalshi market orderbook <ticker>` **(auth)**
Full orderbook depth.
- `--depth <0-100>`

### `kalshi market hot`
Top markets by 24h volume. `--limit <N>` (default 20).

### `kalshi market expiring`
Markets closing soon. `--within <hours>` (default 24).

### `kalshi market spread`
Markets with the widest bid-ask spread. `--limit <N>` (default 20).

### `kalshi market analyze <ticker>`
Orderbook analysis + fill simulation.
- `--buy <qty>` — simulate buying N contracts, show fill price & slippage
- `--sell <qty>` — simulate selling N contracts

---

## Events, series, milestones, collections

### `kalshi event list`
List events. `--status`, `--series-ticker`, `--category`, `--with-nested-markets`.

### `kalshi event get <event_ticker>` / `kalshi event metadata <event_ticker>`
Single event details; `metadata` returns raw JSON metadata.

### `kalshi event multivariate`
List multivariate events.

### `kalshi event candlestick <event_ticker>`
Event-level OHLCV data.

### `kalshi event forecast <event_ticker>`
Forecast percentile history. Pass `--percentile 2500 --percentile 5000 ...` (values in basis points: 2500 = 25th percentile).

### `kalshi series list` / `kalshi series get <series_ticker>`
Browse all series, or one in detail.

### `kalshi series fee-change`
Fee change history for a series.

### `kalshi milestone list` / `kalshi milestone get <milestone_id>`
Sports/event milestones.

### `kalshi collection list` / `kalshi collection get <ticker>`
Multivariate event collections (combo markets).
- `kalshi collection create-market <ticker>` — create market from collection
- `kalshi collection lookup <ticker>` — lookup collection payout
- `kalshi collection lookup-history <ticker>` — payout lookup history

---

## Orders **(auth)**

### `kalshi order create <ticker>`
Full-control order placement.
- `--side yes|no` (required)
- `--action buy|sell` (required)
- `--quantity <N>` (required)
- `--yes-price <cents>` or `--no-price <cents>` — limit price
- `--type gtc|fok|ioc` — time in force (default `gtc`)
- `--post-only` — reject if order would cross the book
- `--reduce-only` — only reduce, never add to a position

### `kalshi order list`
- `--ticker <T>`, `--status resting|canceled|filled`, `--all`

### `kalshi order get <order_id>` / `kalshi order cancel <order_id>`
Single order lookup / cancel.

### `kalshi order amend <order_id>`
Modify a resting order. Requires `--ticker --side --action --quantity` (pass the new values).

### `kalshi order decrease <order_id>`
Reduce order quantity. `--reduce-by <N>`.

### `kalshi order batch-create --file <path>`
Batch create from a JSON file (max 20 orders per request). File format is an array of order objects with the same fields as `order create`.

### `kalshi order batch-cancel`
- `--ticker <T>` — cancel all orders in a market
- `--order-ids <ID1,ID2,...>` — comma-separated list

### `kalshi order queue <ticker>` / `kalshi order queue-position <order_id>`
Check queue positions in the orderbook.

---

## Order groups **(auth)**

Order groups (brackets) let you set a shared max-loss budget across multiple orders.

- `kalshi order-group list`
- `kalshi order-group create --max-loss <cents>`
- `kalshi order-group get <group_id>`
- `kalshi order-group delete <group_id>`
- `kalshi order-group reset <group_id>`
- `kalshi order-group trigger <group_id>`
- `kalshi order-group update-limit <group_id>` — adjust max-loss

---

## RFQs and Quotes **(auth)**

Request-for-quote flow for block trades.

### RFQs
- `kalshi rfq list`
- `kalshi rfq create <ticker> --quantity <N> --side yes|no`
- `kalshi rfq get <rfq_id>` / `kalshi rfq cancel <rfq_id>`
- `kalshi rfq id` — your communications ID

### Quotes
- `kalshi quote list --rfq-id <id>`
- `kalshi quote create --rfq-id <id> --price <cents>`
- `kalshi quote accept <quote_id>`
- `kalshi quote cancel <quote_id>`
- `kalshi quote get <quote_id>` / `kalshi quote confirm <quote_id>`

---

## Portfolio **(auth)**

- `kalshi portfolio balance` — cash balance
- `kalshi portfolio position` — open positions
  - `--ticker`, `--event-ticker`, `--count-filter`, `--settlement-status`, `--all`
- `kalshi portfolio fill` — trade fills
  - `--ticker`, `--order-id`, `--min-ts`, `--max-ts`, `--all`
- `kalshi portfolio settlement` — settlement history
- `kalshi portfolio resting-value` — total value tied up in resting orders
- `kalshi portfolio summary` — aggregate analytics

---

## Historical data

- `kalshi historical market` — past markets (`--ticker`, `--min-close-ts`, `--max-close-ts`)
- `kalshi historical trade` — past trades (`--ticker`, `--min-ts`, `--max-ts`, `--all`)
- `kalshi historical candlestick <ticker>` — historical OHLCV
- `kalshi historical cutoff` — cutoff timestamp (earliest available data)
- `kalshi historical fill` **(auth)** — your past fills
- `kalshi historical order` **(auth)** — your past orders
- `kalshi historical market-detail <ticker>` — single historical market as raw JSON

---

## Export **(auth)**

Write your own data to a file. Every export command takes `-o <path>` and `--format csv|json|jsonl`.

- `kalshi export fill -o fills.csv --format csv --since <unix_ts>`
- `kalshi export position -o positions.json --format json`
- `kalshi export settlement -o settlements.jsonl --format jsonl`

---

## Real-time feeds (WebSocket) **(auth)**

All of these are streaming — they run until Ctrl-C.

- `kalshi watch ticker <TICKER1> [<TICKER2>...]` — live price updates
- `kalshi watch trade [<TICKER>...]` — live trades (all markets if none given)
- `kalshi watch orderbook <TICKER> [...]` — orderbook deltas (`--snapshot` for initial snapshot)
- `kalshi watch fill [<TICKER>...]` — your fill notifications
- `kalshi watch position [<TICKER>...]` — your position updates
- `kalshi watch order [<TICKER>...]` — your order status updates
- `kalshi watch lifecycle` — market & event lifecycle changes
- `kalshi watch communications` — RFQ/quote notifications
- `kalshi watch order-group-updates` — order group updates
- `kalshi watch multivar-lifecycle` — multivariate market lifecycle
- `kalshi watch multivariate` — multivariate collection lookups

---

## Price alerts

Alerts persist locally and fire via native desktop notifications (macOS / Linux).

- `kalshi alert add <ticker> --above <cents>` or `--below <cents>` (can set both)
- `kalshi alert list`
- `kalshi alert remove <alert_id>`
- `kalshi alert watch` — streams WebSocket ticker data and fires notifications when thresholds cross

---

## Utilities

- `kalshi status` **(auth)** — dashboard (also listed under Quick actions above)
- `kalshi ping` — API reachability + latency check (no auth)
- `kalshi shell` — interactive REPL; drop the `kalshi ` prefix from each command
- `kalshi upgrade` — check for and install new version (`--check` to only check)
- `kalshi url <ticker>` — get the kalshi.com URL; `--open` to launch a browser
- `kalshi ticker <url>` — inverse: extract the ticker(s) from a Kalshi website URL
- `kalshi completions <shell>` — shell completion script (`bash`/`zsh`/`fish`/etc.) — hidden command

---

## Exchange, account, config

### Exchange
- `kalshi exchange status` — open/closed/limited
- `kalshi exchange announcement` — platform announcements
- `kalshi exchange schedule` — trading schedule
- `kalshi exchange user-data-timestamp` **(auth)** — last update to your user data

### Account **(auth)**
- `kalshi account limit` — rate limit usage and tier

### Subaccounts **(auth)**
- `kalshi subaccount create --name <n>`
- `kalshi subaccount transfer --from <a> --to <b> --amount <cents>`
- `kalshi subaccount balance` — all subaccount balances
- `kalshi subaccount transfer-list`
- `kalshi subaccount netting` / `kalshi subaccount netting-update`

### Config
- `kalshi config init` — **interactive** setup wizard (user must run — do not script)
- `kalshi config show` — print current effective config
- `kalshi config profile-list` — list named profiles
- `kalshi config profile-add <name>` — add a profile interactively
- `kalshi config profile-remove <name>`

### API keys **(auth)**
- `kalshi api-key list`
- `kalshi api-key create --name <n>`
- `kalshi api-key delete <key_id>`
- `kalshi api-key generate --name <n>` — generates a local key pair and returns the private key

---

## Environment variables

| Variable | Effect |
|---|---|
| `KALSHI_API_KEY_ID` | Override the configured API key ID |
| `KALSHI_PRIVATE_KEY_PATH` | Path to the PEM private key file |
| `KALSHI_PRIVATE_KEY` | Inline private key contents |
| `KALSHI_PROFILE` | Default profile name (equivalent to `--profile`) |
