// API CONSTANTS
pub const API: &str = "/api";
pub const ROOT: &str = "/";
pub const ID: &str = "/:id";
pub const ADMIN: &str = "/admin";
pub const USER: &str = "/user";

// Auth
pub const SIGNUP: &str = "/signup";
pub const SIGNIN: &str = "/signin";

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
pub const FEED_STREAM: &str = "feed";
pub const FEED_MARKET_ORDER: &str = "feed.market.order";
pub const FEED_REMOVE_MARKET: &str = "feed.remove.market";
pub const FEED_CREATE_MARKET: &str = "feed.create.market";
pub const MAX_NATS_RECONNECTS: u8 = 5;
pub const TRADE_STREAM: &str = "trade";
pub const TRADE_UPDATE_ORDER: &str = "trade.update.orders";
pub const RESOLVE_STREAM: &str = "resolve";
pub const RESOLVE_MARKET: &str = "resolve.market";

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
