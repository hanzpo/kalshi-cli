# Common Workflows

End-to-end patterns for typical tasks. Every example assumes a configured CLI (`kalshi config show` returns credentials).

---

## 1. "What's in my account right now?"

```bash
kalshi status
```

One-command dashboard: balance, positions, resting orders, recent fills. For JSON:

```bash
kalshi --output json status | jq .
```

---

## 2. "Find a market to trade on X"

```bash
# Semantic search
kalshi market search "fed rate decision"

# Or browse hot markets
kalshi market hot --limit 20

# Or markets closing soon
kalshi market expiring --within 48
```

Once you have a candidate ticker:

```bash
kalshi market get <TICKER>
kalshi market orderbook <TICKER>       # see the book (auth)
kalshi market analyze <TICKER> --buy 100   # simulate a 100-contract buy
```

---

## 3. "Place a limit buy for 10 YES contracts at 42 cents"

**Always confirm ticker, side, qty, price with the user before executing.**

```bash
# Sanity check the current quote
kalshi market get KXFED-25DEC-T4

# Place the order
kalshi buy KXFED-25DEC-T4 10 --at 42

# Verify it landed
kalshi order list --status resting --ticker KXFED-25DEC-T4
```

For more control (FOK, post-only, etc.):

```bash
kalshi order create KXFED-25DEC-T4 \
  --side yes --action buy --quantity 10 \
  --yes-price 42 --type gtc --post-only
```

---

## 4. "Close my position in X"

```bash
# Quick close at market
kalshi close KXFED-25DEC-T4

# Or see the position first
kalshi portfolio position --ticker KXFED-25DEC-T4
```

---

## 5. "Cancel all my open orders"

```bash
kalshi cancel-all                      # everything
kalshi cancel-all --ticker KXFED-25DEC-T4   # just one market
```

---

## 6. "Watch live prices for a basket of tickers"

```bash
kalshi watch ticker KXFED-25DEC-T4 KXPRES-24-DEM KXPRES-24-GOP
```

Runs until Ctrl-C. For structured output, add `--output json` and pipe.

---

## 7. "Alert me if YES goes above 75¢"

```bash
kalshi alert add KXFED-25DEC-T4 --above 75 --below 25
kalshi alert watch    # streams WS data and fires desktop notifications
```

---

## 8. "Export all my fills to CSV for the tax spreadsheet"

```bash
# All time
kalshi export fill -o fills.csv --format csv

# Since a unix timestamp
kalshi export fill -o 2026-fills.csv --format csv --since 1735689600
```

Also available: `export position`, `export settlement`. Formats: `csv`, `json`, `jsonl`.

---

## 9. "Analyze historical candlestick data"

```bash
# Last N periods of OHLCV
kalshi market candlestick KXFED-25DEC-T4 --series-ticker KXFED --period 60

# Batch across up to 100 tickers
kalshi market candlestick-batch --tickers KXFED-25DEC-T4,KXFED-26JAN-T3
```

For historical (past/closed) markets, use `kalshi historical candlestick <ticker>`.

---

## 10. "Convert between tickers and URLs"

```bash
# Ticker → URL
kalshi url KXMARMAD-26-DUKE
kalshi url KXMARMAD-26-DUKE --open   # open in browser

# URL → ticker (paste a kalshi.com URL)
kalshi ticker https://kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26-duke
```

---

## 11. "Run a bunch of commands without retyping `kalshi`"

```bash
kalshi shell
# Then inside the REPL:
> status
> market hot --limit 5
> portfolio position
> exit
```

---

## 12. "Test an order flow without real money"

```bash
kalshi --demo status
kalshi --demo buy KXFED-25DEC-T4 10 --at 42
kalshi --demo portfolio position
```

Or set `demo = true` in a profile and use `--profile demo`.

---

## 13. "Script something that parses output"

Use `--output json` and pipe to `jq`. Also add `--no-pager --no-color -y` for clean automation.

```bash
# Top 5 tickers by volume, one per line
kalshi --output json --no-pager --no-color market hot --limit 5 \
  | jq -r '.[].ticker'

# All my resting order IDs
kalshi -q order list --status resting
```

The `-q` flag on list commands prints just IDs/tickers, one per line — perfect for `xargs`.

---

## 14. "Close all losing positions quickly"

```bash
# Cancel resting orders first
kalshi cancel-all

# Then close each position (preview first without -y)
kalshi --output json portfolio position --all \
  | jq -r '.[] | select(.unrealized_pnl < 0) | .ticker' \
  | while read -r ticker; do
      kalshi -y close "$ticker"
    done
```

**Dangerous — always dry-run first (omit `-y`, omit the final `while` loop) and show the user the list of tickers before execution.**

---

## 15. "Batch place a pre-computed list of orders"

```bash
# orders.json is an array of up to 20 order objects
kalshi order batch-create --file orders.json
```

Mirror-image to cancel in bulk:

```bash
kalshi order batch-cancel --ticker KXFED-25DEC-T4
kalshi order batch-cancel --order-ids id1,id2,id3
```

---

## 16. "Multiple accounts / environments"

Add a profile:

```bash
kalshi config profile-add trading
# wizard prompts for key id + private key path
```

Use it:

```bash
kalshi --profile trading status
# or
export KALSHI_PROFILE=trading
kalshi status
```

Inspect the whole config:

```bash
kalshi config show
kalshi config profile-list
```

---

## 17. "I'm new — just set me up"

```bash
# 1. Build & install
cargo install --path .

# 2. Configure (interactive — user must run, not the agent)
kalshi config init

# 3. Confirm it works
kalshi ping
kalshi status
```
