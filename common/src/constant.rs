// API CONSTANTS
pub const API: &str = "/api";
pub const ROOT: &str = "/";
pub const ID: &str = "/:id";

// Auth
pub const AUTH: &str = "/auth";
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
pub const MAX_NATS_RECONNECTS: u8 = 5;
