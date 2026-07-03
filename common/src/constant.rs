// API CONSTANTS
pub const API: &str = "/api";
pub const ROOT: &str = "/";
pub const ID: &str = "/:id";
pub const ADMIN: &str = "/admin";
pub const USER: &str = "/user";

// Auth
pub const SIGNUP: &str = "/signup";
pub const SIGNIN: &str = "/signin";
pub const SEND_OTP: &str = "/send-otp";
pub const RESET_PASSWORD: &str = "/reset-password";

// Profile
pub const PROFILE: &str = "/profile";
pub const PICTURE: &str = "/picture";

// Wallet
pub const WALLET: &str = "/wallet";
pub const BALANCE: &str = "/balance";
pub const DEPOSIT: &str = "/deposit";
pub const WITHDRAW: &str = "/withdraw";
pub const TRANSACTIONS: &str = "/transactions";

// Order
pub const ORDER: &str = "/order";
pub const SNAPSHOT: &str = "/snapshot";

// Market
pub const MARKET: &str = "/market";
pub const MARKET_ID: &str = "/:market_id";
pub const RESOLVE: &str = "/resolve";
pub const OUTCOME_ID: &str = "/:outcome_id";

// Holding
pub const HOLDING: &str = "/holding";

// NATS Config
pub const MATCHER_STREAM: &str = "matcher";
pub const MATCHER_PLACE_ORDER: &str = "matcher.place.order";
pub const MATCHER_CANCEL_ORDER: &str = "matcher.cancel.order";
pub const MATCHER_CREATE_MARKET: &str = "matcher.create.market";
pub const MATCHER_REMOVE_MARKET: &str = "matcher.remove.market";
pub const FEED_QUEUE: &str = "feed";
pub const FEED_MARKET_ORDER: &str = "feed.market.order";
pub const FEED_REMOVE_MARKET: &str = "feed.remove.market";
pub const FEED_CREATE_MARKET: &str = "feed.create.market";
pub const MAX_NATS_RECONNECTS: u8 = 5;
pub const TRADE_STREAM: &str = "trade";
pub const TRADE_UPDATE_ORDER: &str = "trade.update.orders";

// Service Ports
pub const AUTH_PORT: u16 = 3001;
pub const USER_PORT: u16 = 3002;
pub const ORDER_PORT: u16 = 3003;
pub const FEED_PORT: u16 = 3004;
pub const ADMIN_PORT: u16 = 3005;
pub const MARKET_PORT: u16 = 3006;

// Redis
pub const MARKET_CACHE_TTL: u64 = 1800; // 30 min
pub const USER_CACHE_TTL: u64 = 3600; // 60 min
pub const OTP_CACHE_TTL: u64 = 60; // 1 min

// Redpanda
pub const AUTO_OFFSET_RESET: &str = "earliest";
pub const ENABLE_AUTO_COMMIT: &str = "true";
pub const AUTO_COMMIT_INTERVAL_MS: &str = "1000";
pub const SESSION_TIMEOUT_MS: &str = "6000";
pub const HOLDING_GROUP_ID: &str = "holding-rust-consumer";
pub const ORDER_GROUP_ID: &str = "order-rust-consumer";
pub const TRADE_GROUP_ID: &str = "trade-rust-consumer";
pub const TRANSACTION_GROUP_ID: &str = "transaction-rust-consumer";
pub const CDC_HOLDING_TOPIC: &str = "polymarket.public.holdings";
pub const CDC_ORDER_TOPIC: &str = "polymarket.public.orders";
pub const CDC_TRADE_TOPIC: &str = "polymarket.public.trades";
pub const CDC_TRANSACTION_TOPIC: &str = "polymarket.public.transactions";
