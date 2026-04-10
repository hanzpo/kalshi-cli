# Kalshi CLI Command Reference

```
kalshi
├── status                          Quick status dashboard (balance, positions, orders)
├── buy <ticker> <qty>              Quick buy contracts
│   ├── --yes / --no                Side (default: yes)
│   └── --at <cents>                Limit price
├── sell <ticker> <qty>             Quick sell contracts
│   ├── --yes / --no                Side (default: yes)
│   └── --at <cents>                Limit price
├── close <ticker>                  Close an open position
├── cancel-all                      Cancel all resting orders
│   └── --ticker                    Filter by market ticker
│
├── market                          Markets: list, search, analytics
│   ├── list                        List markets (paginated)
│   │   ├── --status                Filter: open, closed, settled
│   │   ├── --series-ticker         Filter by series
│   │   ├── --event-ticker          Filter by event
│   │   ├── --include-combos        Include combo/multivariate markets
│   │   └── --all                   Interactive browser (page by page)
│   ├── get <ticker>                Get a single market
│   ├── search <query>              Semantic search (uses Kalshi internal API)
│   │   ├── --limit                 Number of results
│   │   ├── --cursor                Pagination cursor
│   │   └── --all                   Interactive browser
│   ├── trade                       Get market trades
│   │   ├── --ticker                Filter by market
│   │   └── --min-ts / --max-ts     Time range
│   ├── candlestick <ticker>        OHLCV candlestick data
│   │   ├── --series-ticker         Series ticker (required)
│   │   └── --period / --start-ts / --end-ts
│   ├── candlestick-batch           Batch candlesticks (up to 100 tickers)
│   │   └── --tickers               Comma-separated tickers
│   ├── orderbook <ticker>          Orderbook depth (requires auth)
│   │   └── --depth                 Depth 0-100
│   ├── hot                         Hottest markets by 24h volume
│   │   └── --limit                 Number to show (default: 20)
│   ├── expiring                    Markets expiring soon
│   │   └── --within                Hours from now (default: 24)
│   ├── spread                      Markets with widest bid-ask spread
│   │   └── --limit                 Number to show (default: 20)
│   └── analyze <ticker>            Orderbook analysis & fill simulation
│       ├── --buy <qty>             Simulate buying N contracts
│       └── --sell <qty>            Simulate selling N contracts
│
├── event                           Events (groups of related markets)
│   ├── list                        List events (paginated)
│   │   ├── --status / --series-ticker / --category
│   │   └── --with-nested-markets
│   ├── get <event_ticker>          Get a single event
│   ├── metadata <event_ticker>     Get event metadata (JSON)
│   ├── multivariate                List multivariate events
│   ├── candlestick <event_ticker>  Event-level candlesticks
│   └── forecast <event_ticker>     Forecast percentile history
│       └── --percentile 2500 --percentile 5000 ...
│
├── series                          Event series
│   ├── list                        List all series (paginated)
│   ├── get <series_ticker>         Get a single series
│   └── fee-change                  Fee changes for a series
│
├── order                           Order management (requires auth)
│   ├── create <ticker>             Place an order
│   │   ├── --side yes/no           Contract side
│   │   ├── --action buy/sell       Buy or sell
│   │   ├── --quantity              Number of contracts
│   │   ├── --yes-price / --no-price  Limit price in cents
│   │   ├── --type gtc/fok/ioc     Time in force
│   │   └── --post-only / --reduce-only
│   ├── list                        List orders (paginated)
│   │   ├── --ticker / --status     Filters
│   │   └── --all                   Fetch all
│   ├── get <order_id>              Get a single order
│   ├── cancel <order_id>           Cancel an order
│   ├── amend <order_id>            Modify an order
│   ├── decrease <order_id>         Reduce order quantity
│   │   └── --reduce-by
│   ├── batch-create --file <path>  Batch create from JSON (max 20)
│   ├── batch-cancel                Batch cancel orders
│   │   ├── --ticker                By market
│   │   └── --order-ids             Comma-separated IDs
│   ├── queue <ticker>              Get queue positions
│   └── queue-position <order_id>   Queue position for one order
│
├── order-group                     Order groups / brackets (requires auth)
│   ├── list                        List order groups
│   ├── create                      Create an order group
│   │   └── --max-loss              Max loss in cents
│   ├── get <group_id>              Get an order group
│   ├── delete <group_id>           Delete an order group
│   ├── reset <group_id>            Reset an order group
│   ├── trigger <group_id>          Trigger an order group
│   └── update-limit <group_id>     Update max loss
│
├── rfq                             Request for quotes (requires auth)
│   ├── list                        List RFQs
│   ├── create <ticker>             Create an RFQ
│   │   ├── --quantity              Number of contracts
│   │   └── --side                  yes or no
│   ├── get <rfq_id>                Get an RFQ
│   ├── cancel <rfq_id>             Cancel an RFQ
│   └── id                          Get your communications ID
│
├── quote                           Quotes for RFQs (requires auth)
│   ├── list --rfq-id <id>          List quotes for an RFQ
│   ├── create --rfq-id --price     Create a quote
│   ├── accept <quote_id>           Accept a quote
│   ├── cancel <quote_id>           Cancel a quote
│   ├── get <quote_id>              Get a single quote
│   └── confirm <quote_id>          Confirm a quote
│
├── portfolio                       Portfolio data (requires auth)
│   ├── balance                     Account balance
│   ├── position                    Open positions (paginated)
│   │   └── --ticker / --event-ticker / --count-filter / --settlement-status
│   ├── fill                        Trade fills (paginated)
│   │   └── --ticker / --order-id / --min-ts / --max-ts
│   ├── settlement                  Settlement history (paginated)
│   ├── resting-value               Total resting order value
│   └── summary                     Portfolio analytics summary
│
├── historical                      Historical data
│   ├── market                      Past markets (paginated)
│   │   └── --ticker / --min-close-ts / --max-close-ts
│   ├── trade                       Past trades (paginated)
│   │   └── --ticker / --min-ts / --max-ts
│   ├── candlestick <ticker>        Historical candlesticks
│   ├── cutoff                      Cutoff timestamp
│   ├── fill                        Historical fills (requires auth)
│   ├── order                       Historical orders (requires auth)
│   └── market-detail <ticker>      Single historical market (JSON)
│
├── export                          Export data to file (requires auth)
│   ├── fill -o <path>              Export fills
│   │   ├── --format csv/json/jsonl
│   │   └── --since <timestamp>
│   ├── position -o <path>          Export positions
│   └── settlement -o <path>        Export settlements
│
├── watch                           Real-time WebSocket feeds (requires auth)
│   ├── ticker <market> [<market>...]   Live price updates
│   ├── trade [<market>...]         Live trades (all markets if none specified)
│   ├── orderbook <market> [...]    Live orderbook deltas
│   │   └── --snapshot              Request initial orderbook snapshot
│   ├── fill [<market>...]          Your fill notifications
│   ├── position [<market>...]      Your position updates
│   ├── order [<market>...]         Your order status updates
│   ├── lifecycle                   Market & event lifecycle changes
│   ├── communications              RFQ and quote notifications
│   ├── order-group-updates         Order group updates
│   ├── multivar-lifecycle          Multivariate market lifecycle changes
│   └── multivariate                Multivariate collection lookups
│
├── alert                           Price alerts
│   ├── add <ticker>                Set a price alert
│   │   ├── --above <cents>         Alert when price exceeds
│   │   └── --below <cents>         Alert when price drops below
│   ├── list                        List active alerts
│   ├── remove <id>                 Remove an alert
│   └── watch                       Watch alerts via WebSocket
│
├── url <ticker>                    Get Kalshi website URL for a market
│   └── --open                      Open in browser
│
├── exchange                        Exchange info
│   ├── status                      Exchange status
│   ├── announcement                Announcements
│   ├── schedule                    Trading schedule
│   └── user-data-timestamp         User data timestamp (requires auth)
│
├── config                          Configuration management
│   ├── init                        Interactive setup
│   ├── show                        Show current config
│   ├── profile-list                List profiles
│   ├── profile-add <name>          Add a profile
│   └── profile-remove <name>       Remove a profile
│
├── api-key                         API key management (requires auth)
│   ├── list                        List API keys
│   ├── create --name <n>           Create an API key
│   ├── delete <key_id>             Delete an API key
│   └── generate --name <n>         Generate key pair (returns private key)
│
├── account                         Account info (requires auth)
│   └── limit                       Rate limits and usage tier
│
├── subaccount                      Subaccounts (requires auth)
│   ├── create --name <n>           Create a subaccount
│   ├── transfer                    Transfer between subaccounts
│   │   └── --from / --to / --amount
│   ├── balance                     All subaccount balances
│   ├── transfer-list               List transfers
│   ├── netting                     Get netting settings
│   └── netting-update              Update netting settings
│
├── milestone                       Sports & event milestones
│   ├── list --limit <n>            List milestones
│   └── get <milestone_id>          Get a single milestone
│
├── collection                      Multivariate event collections (combos)
│   ├── list                        List collections
│   ├── get <ticker>                Get a single collection
│   ├── create-market <ticker>      Create market from collection
│   ├── lookup <ticker>             Lookup collection payout
│   └── lookup-history <ticker>     Lookup history
│
└── Global flags
    ├── --demo                      Use demo environment
    ├── --config <path>             Config file path
    ├── --output table/json/csv     Output format
    ├── --no-pager                  Disable paging
    ├── --no-color                  Disable colors
    ├── --quiet / -q                Print only IDs/tickers
    ├── --yes / -y                  Skip confirmation prompts
    └── --profile <name>            Config profile to use
```
