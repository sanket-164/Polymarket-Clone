-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enum Types
CREATE TYPE transaction_type AS ENUM ('DEPOSIT', 'WITHDRAW', 'BUY', 'SELL', 'REFUND', 'PAYOUT');
CREATE TYPE market_status AS ENUM ('PENDING', 'ACTIVE', 'CLOSED', 'RESOLVED', 'CANCELLED');
CREATE TYPE order_side AS ENUM ('BUY', 'SELL');
CREATE TYPE order_status AS ENUM ('PENDING', 'PARTIAL', 'FILLED', 'CANCELLED', 'EXPIRED');

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    picture VARCHAR(255),
    mobile_no VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Wallets Table
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE,
    balance DECIMAL(20, 8) DEFAULT 0.00,
    locked_balance DECIMAL(20, 8) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Transactions Table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id UUID NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    type transaction_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON DELETE CASCADE
);

-- Market Table
CREATE TABLE IF NOT EXISTS market (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(255) NOT NULL,
    start_at TIMESTAMP WITH TIME ZONE NOT NULL,
    close_at TIMESTAMP WITH TIME ZONE NOT NULL,
    status market_status DEFAULT 'PENDING' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Outcome Table
CREATE TABLE IF NOT EXISTS outcome (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_id UUID NOT NULL,
    label VARCHAR(255) NOT NULL,
    start_price DECIMAL(20, 8) NOT NULL,
    current_price DECIMAL(20, 8) NOT NULL,
    total_shares DECIMAL(20, 8) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE
);

-- Holdings Table
CREATE TABLE IF NOT EXISTS holdings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    market_id UUID NOT NULL,
    outcome_id UUID NOT NULL,
    shares DECIMAL(20, 8) NOT NULL,
    locked_shares DECIMAL(20, 8) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE,
    FOREIGN KEY (outcome_id) REFERENCES outcome(id) ON DELETE CASCADE
);

-- Orders Table
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    market_id UUID NOT NULL,
    outcome_id UUID NOT NULL,
    side order_side NOT NULL,
    shares DECIMAL(20, 8) NOT NULL,
    remaining_shares DECIMAL(20, 8) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    status order_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE,
    FOREIGN KEY (outcome_id) REFERENCES outcome(id) ON DELETE CASCADE
);

-- Trades table
CREATE TABLE IF NOT EXISTS trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_id UUID NOT NULL,
    outcome_id UUID NOT NULL,
    buy_order_id UUID NOT NULL,
    sell_order_id UUID NOT NULL,
    shares DECIMAL(20, 8) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (buy_order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (sell_order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE,
    FOREIGN KEY (outcome_id) REFERENCES outcome(id) ON DELETE CASCADE
);

-- Resolved Markets Table
CREATE TABLE IF NOT EXISTS resolved_markets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_id UUID UNIQUE NOT NULL,
    winning_outcome_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE,
    FOREIGN KEY (winning_outcome_id) REFERENCES outcome(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS admins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Session
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

ALTER TABLE wallets ADD CONSTRAINT check_balance_non_negative CHECK (balance >= 0 AND locked_balance >= 0);
ALTER TABLE holdings ADD CONSTRAINT unique_user_market_outcome UNIQUE (user_id, market_id, outcome_id);
ALTER TABLE holdings ADD CONSTRAINT check_shares_non_negative CHECK (shares >= 0 AND locked_shares >= 0);
ALTER TABLE orders ADD CONSTRAINT check_shares_price_non_negative CHECK (shares > 0 AND price >= 0);

-- Indexes

-- Users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;

-- Wallets
CREATE INDEX idx_wallets_user_id ON wallets(user_id);

-- Transactions
CREATE INDEX idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_type ON transactions(type);

-- Market
CREATE INDEX idx_market_status ON market(status);
CREATE INDEX idx_market_category ON market(category);
CREATE INDEX idx_market_close_at ON market(close_at);

-- Outcome
CREATE INDEX idx_outcome_market_id ON outcome(market_id);

-- Holdings
CREATE INDEX idx_holdings_user_id ON holdings(user_id);
CREATE INDEX idx_holdings_market_id ON holdings(market_id);
CREATE INDEX idx_holdings_user_market ON holdings(user_id, market_id);
CREATE INDEX idx_holdings_user_market_outcome ON holdings(user_id, market_id, outcome_id);

-- Orders
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_market_id ON orders(market_id);
CREATE INDEX idx_orders_market_outcome ON orders(market_id, outcome_id);
CREATE INDEX idx_orders_status ON orders(status) WHERE status IN ('PENDING', 'PARTIAL');
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);

-- Trades
CREATE INDEX idx_trades_market ON trades(market_id);
CREATE INDEX idx_trades_market_outcome ON trades(market_id, outcome_id);
CREATE INDEX idx_trades_created_at ON trades(created_at DESC);

-- Resolved Markets
CREATE INDEX idx_resolved_markets_market_id ON resolved_markets(market_id);

-- Sessions
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Password: 12345678 (hashed using Argon2id)
INSERT INTO admins (id, name, email, password) VALUES ('00000000-0000-0000-0000-000000000000', 'Admin', 'admin@polymarketclone.com', '$argon2id$v=19$m=19456,t=2,p=1$AaHZRuAc1RJtu7JC9k9Jag$CkH8UBnDyZaPZd1Y2IzEP83F5W03oVdwzQQzESbDzWM');
INSERT INTO users (id, name, email, password) VALUES ('00000000-0000-0000-0000-000000000000', 'User', 'user@polymarketclone.com', '$argon2id$v=19$m=19456,t=2,p=1$AaHZRuAc1RJtu7JC9k9Jag$CkH8UBnDyZaPZd1Y2IzEP83F5W03oVdwzQQzESbDzWM');
INSERT INTO wallets (id, user_id, balance, locked_balance) VALUES ('11111111-1111-1111-1111-111111111111', '00000000-0000-0000-0000-000000000000', 0.00, 0.00);

-- required for Debezium CDC before-state
ALTER TABLE orders REPLICA IDENTITY FULL;
ALTER TABLE holdings REPLICA IDENTITY FULL;