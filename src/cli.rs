use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;


pub const BANNER: &str = "\
\x1b[32m\
\n ██╗  ██╗ █████╗ ██╗     ███████╗██╗  ██╗██╗\
\n ██║ ██╔╝██╔══██╗██║     ██╔════╝██║  ██║██║\
\n █████╔╝ ███████║██║     ███████╗███████║██║\
\n ██╔═██╗ ██╔══██║██║     ╚════██║██╔══██║██║\
\n ██║  ██╗██║  ██║███████╗███████║██║  ██║██║\
\n ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝╚═╝\
\n              Trade on anything.\
\n\x1b[0m";

#[derive(Parser)]
#[command(name = "kalshi", about = "CLI for the Kalshi prediction market API", version, before_help = BANNER)]
pub struct Cli {
    /// Use demo environment
    #[arg(long, global = true)]
    pub demo: bool,

    /// Path to config file (default: ~/.kalshi/config.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Disable automatic paging of long output
    #[arg(long, global = true)]
    pub no_pager: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Quiet mode (print only IDs/tickers, one per line)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    /// Config profile to use
    #[arg(long, global = true)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    // ── Quick actions ──
    /// Quick status dashboard (requires auth)
    Status,
    /// Quick buy (shortcut for order create, requires auth)
    Buy {
        /// Market ticker
        ticker: String,
        /// Number of contracts
        quantity: i64,
        /// Buy YES side (default)
        #[arg(long)]
        yes: bool,
        /// Buy NO side
        #[arg(long)]
        no: bool,
        /// Limit price in cents
        #[arg(long)]
        at: Option<i64>,
    },
    /// Quick sell (shortcut for order create, requires auth)
    Sell {
        /// Market ticker
        ticker: String,
        /// Number of contracts
        quantity: i64,
        /// Sell YES side (default)
        #[arg(long)]
        yes: bool,
        /// Sell NO side
        #[arg(long)]
        no: bool,
        /// Limit price in cents
        #[arg(long)]
        at: Option<i64>,
    },
    /// Close a position (requires auth)
    Close {
        /// Market ticker
        ticker: String,
    },
    /// Cancel all resting orders (requires auth)
    CancelAll {
        /// Filter by ticker
        #[arg(long)]
        ticker: Option<String>,
    },

    // ── Browse & discover ──
    /// Markets (list, search, orderbook, analytics)
    #[command(alias = "markets")]
    Market {
        #[command(subcommand)]
        cmd: MarketCmd,
    },
    /// Events (groups of related markets)
    #[command(alias = "events")]
    Event {
        #[command(subcommand)]
        cmd: EventCmd,
    },
    /// Event series (groups of related events)
    Series {
        #[command(subcommand)]
        cmd: SeriesCmd,
    },
    /// Sports & event milestones
    #[command(alias = "milestones")]
    Milestone {
        #[command(subcommand)]
        cmd: MilestoneCmd,
    },
    /// Multivariate event collections (combo markets)
    #[command(alias = "collections")]
    Collection {
        #[command(subcommand)]
        cmd: CollectionCmd,
    },

    // ── Trading ──
    /// Orders (requires auth)
    #[command(alias = "orders")]
    Order {
        #[command(subcommand)]
        cmd: OrderCmd,
    },
    /// Order groups / brackets (requires auth)
    #[command(alias = "order-groups")]
    OrderGroup {
        #[command(subcommand)]
        cmd: OrderGroupCmd,
    },
    /// Request for quotes / block trades (requires auth)
    #[command(alias = "rfqs")]
    Rfq {
        #[command(subcommand)]
        cmd: RfqCmd,
    },
    /// Quotes for RFQs (requires auth)
    #[command(alias = "quotes")]
    Quote {
        #[command(subcommand)]
        cmd: QuoteCmd,
    },

    // ── Portfolio & account ──
    /// Portfolio: balance, positions, fills, settlements (requires auth)
    Portfolio {
        #[command(subcommand)]
        cmd: PortfolioCmd,
    },
    /// Historical data (past markets, trades, candlesticks)
    Historical {
        #[command(subcommand)]
        cmd: HistoricalCmd,
    },
    /// Export data to CSV/JSON (requires auth)
    Export {
        #[command(subcommand)]
        cmd: ExportCmd,
    },
    /// Account info (requires auth)
    Account {
        #[command(subcommand)]
        cmd: AccountCmd,
    },
    /// Subaccounts (requires auth)
    #[command(alias = "subaccounts")]
    Subaccount {
        #[command(subcommand)]
        cmd: SubaccountCmd,
    },

    // ── Real-time ──
    /// Watch real-time market data via WebSocket (requires auth)
    Watch {
        #[command(subcommand)]
        cmd: WatchCmd,
    },
    /// Price alerts
    #[command(alias = "alerts")]
    Alert {
        #[command(subcommand)]
        cmd: AlertCmd,
    },

    // ── Utilities ──
    /// Get the market ticker(s) from a Kalshi website URL
    Ticker {
        /// Kalshi market URL (e.g. https://kalshi.com/markets/kxmarmad/march-madness-2026/kxmarmad-26)
        url: String,
    },
    /// Get the Kalshi website URL for a market ticker
    Url {
        /// Market ticker (e.g. KXMARMAD-26-DUKE)
        ticker: String,
        /// Open the URL in your browser
        #[arg(long)]
        open: bool,
    },
    /// Exchange status and info
    Exchange {
        #[command(subcommand)]
        cmd: ExchangeCmd,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// API key management (requires auth)
    #[command(alias = "api-keys")]
    ApiKey {
        #[command(subcommand)]
        cmd: ApiKeyCmd,
    },

    /// Check API connectivity and latency (no auth required)
    Ping,
    /// Interactive shell (REPL) for running commands without re-typing `kalshi`
    Shell,
    /// Check for updates and upgrade to the latest version
    Upgrade {
        /// Check only, don't install
        #[arg(long)]
        check: bool,
    },

    // ── Hidden (advanced/niche) ──
    /// Generate shell completions
    #[command(hide = true)]
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Live data feeds for milestones
    #[command(hide = true)]
    LiveData {
        #[command(subcommand)]
        cmd: LiveDataCmd,
    },
    /// Structured targets
    #[command(hide = true)]
    StructuredTarget {
        #[command(subcommand)]
        cmd: StructuredTargetCmd,
    },
    /// Incentive programs
    #[command(hide = true)]
    IncentiveProgram {
        #[command(subcommand)]
        cmd: IncentiveProgramCmd,
    },
    /// FCM data (requires auth)
    #[command(hide = true)]
    Fcm {
        #[command(subcommand)]
        cmd: FcmCmd,
    },
    /// Search tags and filters metadata
    #[command(hide = true)]
    Search {
        #[command(subcommand)]
        cmd: SearchCmd,
    },
}

// ── Config ──

#[derive(Clone, Subcommand)]
pub enum ConfigCmd {
    /// Initialize configuration interactively
    Init,
    /// Show current configuration
    Show,
    /// List config profiles
    ProfileList,
    /// Add a config profile interactively
    ProfileAdd {
        /// Profile name
        name: String,
    },
    /// Remove a config profile
    ProfileRemove {
        /// Profile name
        name: String,
    },
}

// ── Exchange ──

#[derive(Subcommand)]
pub enum ExchangeCmd {
    /// Get exchange status
    Status,
    /// Get exchange announcements
    #[command(alias = "announcements")]
    Announcement,
    /// Get exchange schedule
    Schedule,
    /// Get user data timestamp (requires auth)
    UserDataTimestamp,
}

// ── Market ──

#[derive(Subcommand)]
pub enum MarketCmd {
    /// List markets
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Browse all markets interactively, one page at a time
        #[arg(long)]
        all: bool,
        /// Filter by status (e.g. open, closed, settled)
        #[arg(long)]
        status: Option<String>,
        /// Filter by series ticker
        #[arg(long)]
        series_ticker: Option<String>,
        /// Filter by event ticker
        #[arg(long, alias = "event")]
        event_ticker: Option<String>,
        /// Include combo/multivariate markets (excluded by default)
        #[arg(long)]
        include_combos: bool,
        /// Filter results by keyword (case-insensitive match on title and ticker)
        #[arg(long)]
        search: Option<String>,
    },
    /// Get a single market
    Get {
        /// Market ticker
        ticker: String,
    },
    /// Get market trades
    #[command(alias = "trades")]
    Trade {
        /// Market ticker (optional — omit to list all trades)
        ticker: Option<String>,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Only trades after this unix timestamp
        #[arg(long)]
        min_ts: Option<i64>,
        /// Only trades before this unix timestamp
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get market candlesticks
    #[command(alias = "candlesticks")]
    Candlestick {
        /// Market ticker
        ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Period in minutes (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
        /// Start unix timestamp
        #[arg(long)]
        start_ts: Option<i64>,
        /// End unix timestamp
        #[arg(long)]
        end_ts: Option<i64>,
    },
    /// Get market orderbook (requires auth)
    Orderbook {
        /// Market ticker
        ticker: String,
        /// Depth (0-100)
        #[arg(long)]
        depth: Option<u32>,
    },
    /// Get candlesticks for multiple markets (batch, up to 100)
    CandlestickBatch {
        /// Comma-separated market tickers (up to 100)
        #[arg(long)]
        tickers: String,
        /// Start unix timestamp
        #[arg(long)]
        start_ts: Option<i64>,
        /// End unix timestamp
        #[arg(long)]
        end_ts: Option<i64>,
        /// Period in minutes (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
    },
    /// Search markets by keyword (client-side filter)
    Search {
        /// Search query (matched against title and ticker)
        query: String,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by status (e.g. open, closed, settled)
        #[arg(long)]
        status: Option<String>,
        /// Include combo/multivariate markets (excluded by default)
        #[arg(long)]
        include_combos: bool,
    },
    /// Show hottest markets by volume
    Hot {
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Include combo/multivariate markets (excluded by default)
        #[arg(long)]
        include_combos: bool,
    },
    /// Show markets expiring soon
    Expiring {
        /// Hours from now
        #[arg(long, default_value = "24")]
        within: u64,
        /// Max results to show
        #[arg(long)]
        limit: Option<u32>,
        /// Include combo/multivariate markets (excluded by default)
        #[arg(long)]
        include_combos: bool,
    },
    /// Show markets with widest bid-ask spread
    Spread {
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Include combo/multivariate markets (excluded by default)
        #[arg(long)]
        include_combos: bool,
    },
    /// Analyze orderbook for a market (requires auth)
    Analyze {
        /// Market ticker
        ticker: String,
        /// Simulate buying N contracts
        #[arg(long)]
        buy: Option<i64>,
        /// Simulate selling N contracts
        #[arg(long)]
        sell: Option<i64>,
    },
    /// Price history with human-friendly time ranges (auto-resolves series ticker)
    History {
        /// Market ticker
        ticker: String,
        /// Time interval: 1m, 5m, 1h, 6h, 1d, 1w
        #[arg(long, default_value = "1h")]
        interval: String,
        /// Lookback period: 1d, 1w, 1m, 3m, 1y
        #[arg(long, default_value = "1w")]
        period: String,
    },
    /// Get current prices for multiple markets at once
    Prices {
        /// Market tickers (space-separated)
        #[arg(required = true)]
        tickers: Vec<String>,
    },
    /// Show implied probability distribution for an event's markets
    #[command(alias = "distribution")]
    Dist {
        /// Event ticker (e.g. KXFEDRATE-26APR)
        event_ticker: String,
        /// Show as CDF (cumulative) instead of PDF
        #[arg(long)]
        cdf: bool,
        /// Chart width in columns
        #[arg(long, default_value = "40")]
        width: usize,
        /// Use ask prices (default: midpoint of bid/ask)
        #[arg(long)]
        ask: bool,
        /// Use bid prices (default: midpoint of bid/ask)
        #[arg(long)]
        bid: bool,
    },
}

// ── Event ──

#[derive(Subcommand)]
pub enum EventCmd {
    /// List events
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by status (e.g. open, closed, settled)
        #[arg(long)]
        status: Option<String>,
        /// Filter by series ticker
        #[arg(long)]
        series_ticker: Option<String>,
        /// Filter by category (e.g. "Elections", "Sports", "Financials")
        #[arg(long)]
        category: Option<String>,
        /// Include nested markets in response
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get a single event
    Get {
        /// Event ticker
        event_ticker: String,
        /// Include nested markets in response
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get event metadata
    Metadata {
        /// Event ticker
        event_ticker: String,
    },
    /// List multivariate events
    #[command(alias = "multivariates")]
    Multivariate {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Filter by series ticker
        #[arg(long)]
        series_ticker: Option<String>,
        /// Filter by collection ticker
        #[arg(long)]
        collection_ticker: Option<String>,
        /// Include nested markets in response
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get event candlesticks
    #[command(alias = "candlesticks")]
    Candlestick {
        /// Event ticker
        event_ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Start unix timestamp
        #[arg(long)]
        start_ts: Option<i64>,
        /// End unix timestamp
        #[arg(long)]
        end_ts: Option<i64>,
        /// Period in minutes (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
    },
    /// Get event forecast percentile history (requires auth, numerical events only)
    Forecast {
        /// Event ticker
        event_ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Percentiles (0-10000, e.g. 2500=25th). Repeat for multiple: --percentile 2500 --percentile 5000
        #[arg(long = "percentile", required = true)]
        percentiles: Vec<i64>,
        /// Start timestamp (unix seconds, required)
        #[arg(long)]
        start_ts: i64,
        /// End timestamp (unix seconds, required)
        #[arg(long)]
        end_ts: i64,
        /// Period in minutes: 0 (5-sec), 1, 60, or 1440 (required)
        #[arg(long)]
        period: i64,
    },
}

// ── Series ──

#[derive(Subcommand)]
pub enum SeriesCmd {
    /// List series
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Get a single series
    Get {
        /// Series ticker
        series_ticker: String,
    },
    /// Get fee changes for a series
    FeeChange {
        /// Series ticker
        #[arg(long)]
        series_ticker: Option<String>,
        /// Include historical fee changes
        #[arg(long)]
        show_historical: bool,
    },
}

// ── Order ──

#[derive(Subcommand)]
pub enum OrderCmd {
    /// Place an order
    Create {
        /// Market ticker
        ticker: String,
        /// Side: yes or no
        #[arg(long)]
        side: String,
        /// Action: buy or sell
        #[arg(long)]
        action: String,
        /// Number of contracts
        #[arg(long)]
        quantity: i64,
        /// Yes price in cents (1-99)
        #[arg(long)]
        yes_price: Option<i64>,
        /// No price in cents (1-99)
        #[arg(long)]
        no_price: Option<i64>,
        /// Time in force: gtc, fok, ioc
        #[arg(long, name = "type")]
        tif: Option<String>,
        /// Expiration timestamp (unix seconds)
        #[arg(long)]
        expiration_ts: Option<i64>,
        /// Post only (maker only)
        #[arg(long)]
        post_only: bool,
        /// Reduce only
        #[arg(long)]
        reduce_only: bool,
        /// Client order ID
        #[arg(long)]
        client_order_id: Option<String>,
        /// Order group ID
        #[arg(long)]
        order_group_id: Option<String>,
        /// Max cost in cents (triggers fill-or-kill)
        #[arg(long)]
        buy_max_cost: Option<i64>,
    },
    /// List orders
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Filter by status (e.g. resting, executed, canceled)
        #[arg(long)]
        status: Option<String>,
    },
    /// Get a single order
    Get {
        /// Order ID
        order_id: String,
    },
    /// Cancel an order
    Cancel {
        /// Order ID
        order_id: String,
    },
    /// Amend an order
    Amend {
        /// Order ID
        order_id: String,
        /// Market ticker
        #[arg(long)]
        ticker: String,
        /// Side: yes or no
        #[arg(long)]
        side: String,
        /// Action: buy or sell
        #[arg(long)]
        action: String,
        /// New quantity
        #[arg(long)]
        quantity: Option<i64>,
        /// New yes price in cents
        #[arg(long)]
        yes_price: Option<i64>,
        /// New no price in cents
        #[arg(long)]
        no_price: Option<i64>,
    },
    /// Decrease order quantity
    Decrease {
        /// Order ID
        order_id: String,
        /// Amount to reduce by
        #[arg(long)]
        reduce_by: i64,
    },
    /// Batch create orders from JSON file (max 20)
    BatchCreate {
        /// Path to JSON file with order array
        #[arg(long)]
        file: PathBuf,
    },
    /// Batch cancel orders
    BatchCancel {
        /// Filter by ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Specific order IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        order_ids: Option<Vec<String>>,
    },
    /// Get queue positions
    Queue {
        /// Market ticker
        ticker: String,
    },
    /// Get queue position for a single order
    QueuePosition {
        /// Order ID
        order_id: String,
    },
}

// ── Order Group ──

#[derive(Subcommand)]
pub enum OrderGroupCmd {
    /// List order groups
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Create an order group
    Create {
        /// Max loss in cents
        #[arg(long)]
        max_loss: Option<i64>,
    },
    /// Get an order group
    Get {
        /// Order group ID
        group_id: String,
    },
    /// Delete an order group
    Delete {
        /// Order group ID
        group_id: String,
    },
    /// Reset an order group
    Reset {
        /// Order group ID
        group_id: String,
    },
    /// Trigger an order group
    Trigger {
        /// Order group ID
        group_id: String,
    },
    /// Update order group max loss
    UpdateLimit {
        /// Order group ID
        group_id: String,
        /// New max loss in cents
        #[arg(long)]
        max_loss: i64,
    },
}

// ── Portfolio ──

#[derive(Subcommand)]
pub enum PortfolioCmd {
    /// Get account balance
    Balance,
    /// Get open positions
    #[command(alias = "positions")]
    Position {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Filter by event ticker
        #[arg(long, alias = "event")]
        event_ticker: Option<String>,
        /// Filter by count (e.g. gt:0)
        #[arg(long)]
        count_filter: Option<String>,
        /// Filter by settlement status (e.g. unsettled, settled)
        #[arg(long)]
        settlement_status: Option<String>,
    },
    /// Get trade fills
    #[command(alias = "fills")]
    Fill {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Filter by order ID
        #[arg(long)]
        order_id: Option<String>,
        /// Only fills after this unix timestamp
        #[arg(long)]
        min_ts: Option<i64>,
        /// Only fills before this unix timestamp
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get settlement history
    #[command(alias = "settlements")]
    Settlement {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Get total resting order value
    RestingValue,
    /// Portfolio analytics summary
    Summary,
}

// ── Historical ──

#[derive(Subcommand)]
pub enum HistoricalCmd {
    /// List historical markets
    #[command(alias = "markets")]
    Market {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Only markets closed after this unix timestamp
        #[arg(long)]
        min_close_ts: Option<i64>,
        /// Only markets closed before this unix timestamp
        #[arg(long)]
        max_close_ts: Option<i64>,
    },
    /// List trades on settled markets (for active market trades, use `market trade`)
    #[command(alias = "trades")]
    Trade {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Only trades after this unix timestamp
        #[arg(long)]
        min_ts: Option<i64>,
        /// Only trades before this unix timestamp
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get historical candlesticks
    #[command(alias = "candlesticks")]
    Candlestick {
        /// Market ticker
        ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Period in minutes (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
        /// Start unix timestamp
        #[arg(long)]
        start_ts: Option<i64>,
        /// End unix timestamp
        #[arg(long)]
        end_ts: Option<i64>,
    },
    /// Get cutoff timestamps
    Cutoff,
    /// Get historical fills (requires auth)
    #[command(alias = "fills")]
    Fill {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Get historical orders (requires auth)
    #[command(alias = "orders")]
    Order {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Get a single historical market
    MarketDetail {
        /// Market ticker
        ticker: String,
    },
}

// ── Subaccount ──

#[derive(Subcommand)]
pub enum SubaccountCmd {
    /// Create a subaccount
    Create {
        /// Subaccount name
        #[arg(long)]
        name: String,
    },
    /// Transfer between subaccounts
    Transfer {
        /// Source subaccount ID
        #[arg(long)]
        from: i64,
        /// Destination subaccount ID
        #[arg(long)]
        to: i64,
        /// Amount in cents
        #[arg(long)]
        amount: i64,
    },
    /// Get all subaccount balances
    Balance,
    /// List transfers
    #[command(alias = "transfers")]
    TransferList {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Get netting settings
    Netting,
    /// Update netting settings
    NettingUpdate {
        /// Subaccount number
        #[arg(long)]
        subaccount_number: i64,
        /// Enable or disable netting
        #[arg(long)]
        enabled: bool,
    },
}

// ── API Key ──

#[derive(Subcommand)]
pub enum ApiKeyCmd {
    /// List API keys
    List,
    /// Create an API key
    Create {
        /// Key name
        #[arg(long)]
        name: String,
    },
    /// Delete an API key
    Delete {
        /// API key ID
        key_id: String,
    },
    /// Generate an API key pair (returns private key)
    Generate {
        /// Key name
        #[arg(long)]
        name: String,
        /// Comma-separated scopes
        #[arg(long)]
        scopes: Option<String>,
    },
}

// ── RFQ ──

#[derive(Subcommand)]
pub enum RfqCmd {
    /// List RFQs
    List {
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Create an RFQ
    Create {
        /// Market ticker
        ticker: String,
        /// Number of contracts
        #[arg(long)]
        quantity: i64,
        /// Side: yes or no
        #[arg(long)]
        side: String,
    },
    /// Get an RFQ
    Get {
        /// RFQ ID
        rfq_id: String,
    },
    /// Cancel an RFQ
    Cancel {
        /// RFQ ID
        rfq_id: String,
    },
    /// Get your communications ID
    Id,
}

// ── Quote ──

#[derive(Subcommand)]
pub enum QuoteCmd {
    /// List quotes
    List {
        /// RFQ ID
        #[arg(long)]
        rfq_id: String,
    },
    /// Create a quote
    Create {
        /// RFQ ID
        #[arg(long)]
        rfq_id: String,
        /// Price in cents
        #[arg(long)]
        price: i64,
    },
    /// Accept a quote
    Accept {
        /// Quote ID
        quote_id: String,
    },
    /// Cancel a quote
    Cancel {
        /// Quote ID
        quote_id: String,
    },
    /// Get a single quote
    Get {
        /// Quote ID
        quote_id: String,
    },
    /// Confirm a quote
    Confirm {
        /// Quote ID
        quote_id: String,
    },
}

// ── Account ──

#[derive(Subcommand)]
pub enum AccountCmd {
    /// Get account rate limits and usage tier
    Limit,
}

// ── Search ──

#[derive(Subcommand)]
pub enum SearchCmd {
    /// Get tags by categories
    Tag,
    /// Get filters by sport
    Filter,
}

// ── Milestone ──

#[derive(Subcommand)]
pub enum MilestoneCmd {
    /// List milestones
    List {
        /// Number of results (1-500)
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Only milestones starting after this date (YYYY-MM-DD)
        #[arg(long)]
        minimum_start_date: Option<String>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter by competition
        #[arg(long)]
        competition: Option<String>,
        /// Filter by source ID
        #[arg(long)]
        source_id: Option<String>,
        /// Filter by milestone type
        #[arg(long, name = "type")]
        milestone_type: Option<String>,
        /// Filter by related event ticker
        #[arg(long)]
        related_event_ticker: Option<String>,
        /// Only milestones updated after this unix timestamp
        #[arg(long)]
        min_updated_ts: Option<i64>,
    },
    /// Get a single milestone
    Get {
        /// Milestone ID
        milestone_id: String,
    },
}

// ── Live Data ──

#[derive(Subcommand)]
pub enum LiveDataCmd {
    /// Get live data for a milestone
    Get {
        /// Milestone ID
        milestone_id: String,
        /// Data type
        #[arg(long, name = "type")]
        data_type: String,
    },
    /// Batch get live data for multiple milestones
    Batch {
        /// Comma-separated milestone IDs (max 100)
        #[arg(long)]
        milestone_ids: String,
    },
}

// ── Structured Target ──

#[derive(Subcommand)]
pub enum StructuredTargetCmd {
    /// List structured targets
    List {
        /// Filter by target type
        #[arg(long, name = "type")]
        target_type: Option<String>,
        /// Filter by competition
        #[arg(long)]
        competition: Option<String>,
        /// Results per page
        #[arg(long)]
        page_size: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Get a single structured target
    Get {
        /// Structured target ID
        id: String,
    },
}

// ── Incentive Program ──

#[derive(Subcommand)]
pub enum IncentiveProgramCmd {
    /// List incentive programs
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by program type
        #[arg(long, name = "type")]
        program_type: Option<String>,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
}

// ── FCM ──

#[derive(Subcommand)]
pub enum FcmCmd {
    /// List FCM orders
    #[command(alias = "orders")]
    Order {
        /// Subtrader ID (required)
        #[arg(long)]
        subtrader_id: String,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
        /// Filter by event ticker
        #[arg(long, alias = "event")]
        event_ticker: Option<String>,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Only orders after this unix timestamp
        #[arg(long)]
        min_ts: Option<i64>,
        /// Only orders before this unix timestamp
        #[arg(long)]
        max_ts: Option<i64>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
    },
    /// List FCM positions
    #[command(alias = "positions")]
    Position {
        /// Subtrader ID (required)
        #[arg(long)]
        subtrader_id: String,
        /// Filter by market ticker
        #[arg(long)]
        ticker: Option<String>,
        /// Filter by event ticker
        #[arg(long, alias = "event")]
        event_ticker: Option<String>,
        /// Filter by count (e.g. gt:0)
        #[arg(long)]
        count_filter: Option<String>,
        /// Filter by settlement status (e.g. unsettled, settled)
        #[arg(long)]
        settlement_status: Option<String>,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
}

// ── Collection (Multivariate Event Collections) ──

#[derive(Subcommand)]
pub enum CollectionCmd {
    /// List collections
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by associated event ticker
        #[arg(long)]
        associated_event_ticker: Option<String>,
        /// Filter by series ticker
        #[arg(long)]
        series_ticker: Option<String>,
        /// Max results to return
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },
    /// Get a single collection
    Get {
        /// Collection ticker
        ticker: String,
    },
    /// Create market from collection (requires auth)
    CreateMarket {
        /// Collection ticker
        ticker: String,
        /// Path to JSON file with selected_markets array
        #[arg(long)]
        file: PathBuf,
        /// Include market payload in response
        #[arg(long)]
        with_market_payload: bool,
    },
    /// Lookup collection payout (requires auth)
    Lookup {
        /// Collection ticker
        ticker: String,
        /// Path to JSON file with selected_markets array
        #[arg(long)]
        file: PathBuf,
    },
    /// Get lookup history
    LookupHistory {
        /// Collection ticker
        ticker: String,
        /// Lookback seconds (10, 60, 300, or 3600)
        #[arg(long)]
        lookback_seconds: Option<i64>,
    },
}

// ── Export ──

#[derive(Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Csv,
    Json,
    Jsonl,
}

#[derive(Subcommand)]
pub enum ExportCmd {
    /// Export trade fills
    #[command(alias = "fills")]
    Fill {
        /// Output format
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        /// Only fills after this unix timestamp
        #[arg(long)]
        since: Option<i64>,
        /// Output file path
        #[arg(short = 'o', long = "file")]
        file: PathBuf,
    },
    /// Export positions
    #[command(alias = "positions")]
    Position {
        /// Output format
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        /// Output file path
        #[arg(short = 'o', long = "file")]
        file: PathBuf,
    },
    /// Export settlements
    #[command(alias = "settlements")]
    Settlement {
        /// Output format
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        /// Output file path
        #[arg(short = 'o', long = "file")]
        file: PathBuf,
    },
}

// ── Watch ──

#[derive(Subcommand)]
pub enum WatchCmd {
    /// Watch live price updates for one or more markets (requires auth)
    #[command(alias = "tickers")]
    Ticker {
        /// Market ticker(s)
        #[arg(required = true)]
        markets: Vec<String>,
    },
    /// Watch live trades (optionally filtered to specific markets, requires auth)
    #[command(alias = "trades")]
    Trade {
        /// Market ticker(s) (omit to watch all trades)
        markets: Vec<String>,
    },
    /// Watch orderbook updates for one or more markets (requires auth)
    #[command(alias = "orderbooks")]
    Orderbook {
        /// Market ticker(s)
        #[arg(required = true)]
        markets: Vec<String>,
        /// Request an initial orderbook snapshot before streaming deltas
        #[arg(long)]
        snapshot: bool,
    },
    /// Watch your fill notifications (requires auth)
    #[command(alias = "fills")]
    Fill {
        /// Market ticker(s) (omit to watch all fills)
        markets: Vec<String>,
    },
    /// Watch your position updates (requires auth)
    #[command(alias = "positions")]
    Position {
        /// Market ticker(s) (omit to watch all positions)
        markets: Vec<String>,
    },
    /// Watch your order status updates (requires auth)
    #[command(alias = "orders")]
    Order {
        /// Market ticker(s) (omit to watch all orders)
        markets: Vec<String>,
    },
    /// Watch market & event lifecycle changes (created, settled, etc., requires auth)
    Lifecycle,
    /// Watch RFQ and quote notifications (requires auth)
    Communications,
    /// Watch order group updates (requires auth)
    OrderGroupUpdates,
    /// Watch multivariate market & event lifecycle changes (requires auth)
    MultivarLifecycle,
    /// Watch multivariate collection lookups (requires auth)
    Multivariate,
}

// ── Alert ──

#[derive(Subcommand)]
pub enum AlertCmd {
    /// Add a price alert
    Add {
        /// Market ticker
        ticker: String,
        /// Alert when price goes above (cents)
        #[arg(long)]
        above: Option<f64>,
        /// Alert when price goes below (cents)
        #[arg(long)]
        below: Option<f64>,
    },
    /// List active alerts
    List,
    /// Remove an alert
    Remove {
        /// Alert ID (first 8 chars is enough)
        id: String,
    },
    /// Watch alerts via WebSocket (requires auth)
    Watch,
}
