
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS trading_pairs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) UNIQUE NOT NULL,
    base_asset VARCHAR(10) NOT NULL,
    quote_asset VARCHAR(10) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    asset VARCHAR(10) NOT NULL,
    available DECIMAL(20,8) NOT NULL DEFAULT 0,
    locked DECIMAL(20,8) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, asset)
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    order_type VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20,8) NOT NULL,
    price DECIMAL(20,8),
    filled_quantity DECIMAL(20,8) NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trades (
    id UUID PRIMARY KEY,
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    buyer_order_id UUID NOT NULL REFERENCES orders(id),
    seller_order_id UUID NOT NULL REFERENCES orders(id),
    buyer_user_id UUID NOT NULL REFERENCES users(id),
    seller_user_id UUID NOT NULL REFERENCES users(id),
    price DECIMAL(20,8) NOT NULL,
    quantity DECIMAL(20,8) NOT NULL,
    volume DECIMAL(30,8) NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_trades_executed_at ON trades(executed_at);
CREATE INDEX IF NOT EXISTS idx_balances_user_id ON balances(user_id);

INSERT INTO trading_pairs (symbol, base_asset, quote_asset)
VALUES ('BTC_USD', 'BTC', 'USD')
ON CONFLICT (symbol) DO NOTHING;

INSERT INTO users (id, username) 
VALUES 
    ('123e4567-e89b-12d3-a456-426614174000', 'user123'),
    ('456e7890-e89b-12d3-a456-426614174001', 'user456')
ON CONFLICT (username) DO NOTHING;


INSERT INTO balances (user_id, asset, available) VALUES
    ('123e4567-e89b-12d3-a456-426614174000', 'BTC', 10.00000000),
    ('123e4567-e89b-12d3-a456-426614174000', 'USD', 100000.00),
    ('456e7890-e89b-12d3-a456-426614174001', 'BTC', 5.00000000),
    ('456e7890-e89b-12d3-a456-426614174001', 'USD', 50000.00)
ON CONFLICT (user_id, asset) DO NOTHING;