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
pub const NATS_STREAM: &str = "nats";
pub const SUBJECT_PLACE_ORDER: &str = "nats.place.order";
pub const SUBJECT_CENCEL_ORDER: &str = "nats.cancel.order";
pub const SUBJECT_CREATE_MARKET: &str = "nats.create.market";
pub const SUBJECT_REMOVE_MARKET: &str = "nats.remove.market";
pub const MAX_RECONNECTS: u8 = 5;
