use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(name = "kalshi", about = "CLI for the Kalshi prediction market API", version)]
pub struct Cli {
    /// Use demo/sandbox environment
    #[arg(long, global = true)]
    pub demo: bool,

    /// Path to config file (default: ~/.kalshi/config.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Verbose output (show request details)
    #[arg(short, long, global = true)]
    pub verbose: bool,

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
    /// Manage configuration
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Exchange status and info
    Exchange {
        #[command(subcommand)]
        cmd: ExchangeCmd,
    },
    /// Markets
    Market {
        #[command(subcommand)]
        cmd: MarketCmd,
    },
    /// Events
    Event {
        #[command(subcommand)]
        cmd: EventCmd,
    },
    /// Series
    Series {
        #[command(subcommand)]
        cmd: SeriesCmd,
    },
    /// Orders (requires auth)
    Order {
        #[command(subcommand)]
        cmd: OrderCmd,
    },
    /// Order groups (requires auth)
    OrderGroup {
        #[command(subcommand)]
        cmd: OrderGroupCmd,
    },
    /// Portfolio (requires auth)
    Portfolio {
        #[command(subcommand)]
        cmd: PortfolioCmd,
    },
    /// Historical data
    Historical {
        #[command(subcommand)]
        cmd: HistoricalCmd,
    },
    /// Subaccounts (requires auth)
    Subaccount {
        #[command(subcommand)]
        cmd: SubaccountCmd,
    },
    /// API key management (requires auth)
    ApiKey {
        #[command(subcommand)]
        cmd: ApiKeyCmd,
    },
    /// Request for quotes (requires auth)
    Rfq {
        #[command(subcommand)]
        cmd: RfqCmd,
    },
    /// Quotes (requires auth)
    Quote {
        #[command(subcommand)]
        cmd: QuoteCmd,
    },
    /// Account info (requires auth)
    Account {
        #[command(subcommand)]
        cmd: AccountCmd,
    },
    /// Search metadata
    Search {
        #[command(subcommand)]
        cmd: SearchCmd,
    },
    /// Milestones
    Milestone {
        #[command(subcommand)]
        cmd: MilestoneCmd,
    },
    /// Live data feeds
    LiveData {
        #[command(subcommand)]
        cmd: LiveDataCmd,
    },
    /// Structured targets
    StructuredTarget {
        #[command(subcommand)]
        cmd: StructuredTargetCmd,
    },
    /// Incentive programs
    IncentiveProgram {
        #[command(subcommand)]
        cmd: IncentiveProgramCmd,
    },
    /// FCM data (requires auth)
    Fcm {
        #[command(subcommand)]
        cmd: FcmCmd,
    },
    /// Multivariate event collections
    Collection {
        #[command(subcommand)]
        cmd: CollectionCmd,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
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
    /// Export data to file (requires auth)
    Export {
        #[command(subcommand)]
        cmd: ExportCmd,
    },
    /// Watch real-time market data via WebSocket
    Watch {
        #[command(subcommand)]
        cmd: WatchCmd,
    },
    /// Price alerts
    Alert {
        #[command(subcommand)]
        cmd: AlertCmd,
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
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        /// Browse all markets interactively, one page at a time
        #[arg(long)]
        all: bool,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
    },
    /// Get a single market
    Get {
        /// Market ticker
        ticker: String,
    },
    /// Get market trades
    Trade {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get market candlesticks
    Candlestick {
        /// Market ticker
        ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Period (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
        #[arg(long)]
        start_ts: Option<i64>,
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
        #[arg(long)]
        start_ts: Option<i64>,
        #[arg(long)]
        end_ts: Option<i64>,
        /// Period (e.g. 1, 60, 1440)
        #[arg(long)]
        period: Option<i64>,
    },
    /// Search markets by keyword (client-side filter)
    Search {
        /// Search query (matched against title and ticker)
        query: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Show hottest markets by volume
    Hot {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Show markets expiring soon
    Expiring {
        /// Hours from now
        #[arg(long, default_value = "24")]
        within: u64,
    },
    /// Show markets with widest bid-ask spread
    Spread {
        #[arg(long, default_value = "20")]
        limit: u32,
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
}

// ── Event ──

#[derive(Subcommand)]
pub enum EventCmd {
    /// List events
    List {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        /// Filter by category (e.g. "Elections", "Sports", "Financials")
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get a single event
    Get {
        /// Event ticker
        event_ticker: String,
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get event metadata
    Metadata {
        /// Event ticker
        event_ticker: String,
    },
    /// List multivariate events
    Multivariate {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[arg(long)]
        collection_ticker: Option<String>,
        #[arg(long)]
        with_nested_markets: bool,
    },
    /// Get event candlesticks
    Candlestick {
        /// Event ticker
        event_ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        #[arg(long)]
        start_ts: Option<i64>,
        #[arg(long)]
        end_ts: Option<i64>,
        /// Period (e.g. 1, 60, 1440)
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
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        ticker: Option<String>,
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
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
    Position {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        count_filter: Option<String>,
        #[arg(long)]
        settlement_status: Option<String>,
    },
    /// Get trade fills
    Fill {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get settlement history
    Settlement {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        all: bool,
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
    Market {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        min_close_ts: Option<i64>,
        #[arg(long)]
        max_close_ts: Option<i64>,
    },
    /// List historical trades
    Trade {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
    },
    /// Get historical candlesticks
    Candlestick {
        /// Market ticker
        ticker: String,
        /// Series ticker
        #[arg(long)]
        series_ticker: String,
        /// Period
        #[arg(long)]
        period: Option<i64>,
        #[arg(long)]
        start_ts: Option<i64>,
        #[arg(long)]
        end_ts: Option<i64>,
    },
    /// Get cutoff timestamps
    Cutoff,
    /// Get historical fills (requires auth)
    Fill {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Get historical orders (requires auth)
    Order {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
    TransferList {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        minimum_start_date: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        competition: Option<String>,
        #[arg(long)]
        source_id: Option<String>,
        #[arg(long, name = "type")]
        milestone_type: Option<String>,
        #[arg(long)]
        related_event_ticker: Option<String>,
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
        #[arg(long, name = "type")]
        target_type: Option<String>,
        #[arg(long)]
        competition: Option<String>,
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
        #[arg(long)]
        status: Option<String>,
        #[arg(long, name = "type")]
        program_type: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
}

// ── FCM ──

#[derive(Subcommand)]
pub enum FcmCmd {
    /// List FCM orders
    Order {
        /// Subtrader ID (required)
        #[arg(long)]
        subtrader_id: String,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// List FCM positions
    Position {
        /// Subtrader ID (required)
        #[arg(long)]
        subtrader_id: String,
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        count_filter: Option<String>,
        #[arg(long)]
        settlement_status: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
}

// ── Collection (Multivariate Event Collections) ──

#[derive(Subcommand)]
pub enum CollectionCmd {
    /// List collections
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        associated_event_ticker: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
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
    Fill {
        /// Output format
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        /// Only fills after this unix timestamp
        #[arg(long)]
        since: Option<i64>,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Export positions
    Position {
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Export settlements
    Settlement {
        #[arg(long, default_value = "csv")]
        format: ExportFormat,
        #[arg(short, long)]
        output: PathBuf,
    },
}

// ── Watch ──

#[derive(Subcommand)]
pub enum WatchCmd {
    /// Watch live price updates for a market
    Ticker {
        /// Market ticker
        market: String,
    },
    /// Watch live trades for a market
    Trade {
        /// Market ticker
        market: String,
    },
    /// Watch your fill notifications (requires auth)
    Fill,
    /// Watch your position updates (requires auth)
    Position,
    /// Watch orderbook updates for a market (requires auth)
    Orderbook {
        /// Market ticker
        market: String,
    },
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
    /// Watch alerts via WebSocket
    Watch,
}
