// API CONSTANTS
// API
pub const API_PREFIX: &str = "/api";

// Health
pub const HEALTH_CHECK: &str = "/";

// Auth
pub const AUTH_PREFIX: &str = "/auth";
pub const AUTH_SIGNUP: &str = "/signup";
pub const AUTH_SIGNIN: &str = "/signin";

// Profile
pub const PROFILE_PREFIX: &str = "/profile";
pub const PROFILE_GET_ME: &str = "/";
pub const PROFILE_UPDATE: &str = "/";
pub const PROFILE_UPDATE_PICTURE: &str = "/picture";

// Wallet
pub const WALLET_PREFIX: &str = "/wallet";
pub const WALLET_BALANCE: &str = "/balance";
pub const WALLET_DEPOSIT: &str = "/deposit";
pub const WALLET_WITHDRAW: &str = "/withdraw";
pub const WALLET_TRANSACTIONS: &str = "/transactions";

// Order
pub const ORDER_PREFIX: &str = "/order";
pub const ORDER_GET: &str = "/";
pub const ORDER_PLACE: &str = "/";

// NATS Config
pub const NATS_STREAM: &str = "nats";
pub const SUBJECT_INSERT_ORDER: &str = "nats.insert.order";
pub const SUBJECT_CENCEL_ORDER: &str = "nats.cancel.order";
pub const SUBJECT_INSERT_MARKET: &str = "nats.insert.market";
pub const SUBJECT_REMOVE_MARKET: &str = "nats.remove.market";
pub const MAX_RECONNECTS: u8 = 5;
