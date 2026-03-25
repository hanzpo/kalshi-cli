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
}

// ── Config ──

#[derive(Clone, Subcommand)]
pub enum ConfigCmd {
    /// Initialize configuration interactively
    Init,
    /// Show current configuration
    Show,
}

// ── Exchange ──

#[derive(Subcommand)]
pub enum ExchangeCmd {
    /// Get exchange status
    Status,
    /// Get exchange announcements
    Announcements,
    /// Get exchange schedule
    Schedule,
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
    Trades {
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
    Candlesticks {
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
    Positions {
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
    Fills {
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
    Settlements {
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
}

// ── Historical ──

#[derive(Subcommand)]
pub enum HistoricalCmd {
    /// List historical markets
    Markets {
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
    Trades {
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
    Candlesticks {
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
    Fills {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Get historical orders (requires auth)
    Orders {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        ticker: Option<String>,
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
    Balances,
    /// List transfers
    Transfers {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get netting settings
    Netting,
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
}
