-- Your SQL goes here

-- market configs
CREATE TABLE market_configs (
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,

  max_taker_fee NUMERIC NOT NULL,
  min_taker_fee NUMERIC NOT NULL,
  max_maker_fee NUMERIC NOT NULL,
  min_maker_fee NUMERIC NOT NULL,
  liquidation_fee NUMERIC NOT NULL,

  min_funding_rate NUMERIC NOT NULL,
  max_funding_rate NUMERIC NOT NULL,
  pool_funding_discount NUMERIC NOT NULL,
  funding_interval NUMERIC NOT NULL,

  max_oi NUMERIC NOT NULL,
  max_oi_imbalance NUMERIC NOT NULL,

  max_leverage NUMERIC NOT NULL,
  base_maintenance_margin NUMERIC NULL,
  base_position_limit NUMERIC NOT NULL,
  max_position_limit NUMERIC NOT NULL,

  min_order_size NUMERIC NOT NULL,

  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    type_hash
  )
);
CREATE INDEX mc_tv_mt_pt on market_configs (transaction_version, margin_type, perp_type);

-- market infos
CREATE TABLE markets (
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,

  long_margin_base NUMERIC NOT NULL,
  long_margin_elastic NUMERIC NOT NULL,
  short_margin_base NUMERIC NOT NULL,
  short_margin_elastic NUMERIC NOT NULL,

  long_oi NUMERIC NOT NULL,
  short_oi NUMERIC NOT NULL,

  next_funding_rate NUMERIC NOT NULL,
  next_funding_pos BOOLEAN NOT NULL,
  last_funding_round NUMERIC NOT NULL,

  is_long_close_only BOOLEAN NOT NULL,
  is_short_close_only BOOLEAN NOT NULL,

  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    type_hash
  )
);
CREATE INDEX markets_tv_mt_pt on markets (transaction_version, margin_type, perp_type);

-- limit order infos
CREATE TABLE limit_orders (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  id NUMERIC NOT NULL,
  is_long BOOLEAN NOT NULL,
  is_increase BOOLEAN NOT NULL,
  position_size NUMERIC NOT NULL,
  margin NUMERIC NOT NULL,
  trigger_price NUMERIC NOT NULL,
  triggers_above BOOLEAN NOT NULL,
  trigger_payment NUMERIC NOT NULL,
  max_price_slippage NUMERIC NOT NULL,
  expiration NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    id
  )
);
CREATE INDEX lo_tv_ua_mt_pt on limit_orders (transaction_version, user_addr, margin_type, perp_type);

-- open limit orders
CREATE TABLE open_limit_orders (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  id NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    id
  )
);
CREATE INDEX ol_tv_ua_mt_pt on open_limit_orders (user_addr, margin_type, perp_type);

-- closed limit orders
CREATE TABLE closed_limit_orders (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  id NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    id
  )
);
CREATE INDEX cl_tv_ua_mt_pt on closed_limit_orders (user_addr, margin_type, perp_type);

-- positions
CREATE TABLE positions (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  id NUMERIC NOT NULL,
  opening_price NUMERIC NOT NULL,
  is_long BOOLEAN NOT NULL,
  margin_part NUMERIC NOT NULL,
  position_size NUMERIC NOT NULL,
  maintenance_margin NUMERIC NOT NULL,
  take_profit_price NUMERIC NOT NULL,
  stop_loss_price NUMERIC NOT NULL,
  trigger_payment NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    id
  )
);
CREATE INDEX positions_tv_ua_mt_pt on positions (transaction_version, user_addr, margin_type, perp_type);
CREATE INDEX positions_tv_ua on positions (user_addr);

-- trades
CREATE TABLE trades (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  id NUMERIC NOT NULL,
  is_long BOOLEAN NOT NULL,
  size NUMERIC NOT NULL,
  price NUMERIC NOT NULL,
  fee NUMERIC NOT NULL,
  pnl NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    id,
    transaction_version
  )
);
CREATE INDEX trades_ua_mt_pt on trades (user_addr, margin_type, perp_type);
CREATE INDEX trades_ua on trades (user_addr);

CREATE VIEW user_trades AS 
SELECT 
    user_addr, 
    total_pnl, 
    total_fee, 
    trade_count, 
    profit, 
    volume, 
    RANK() OVER (ORDER BY profit DESC) AS rank
FROM (
    SELECT 
        user_addr, 
        SUM(pnl) AS total_pnl, 
        SUM(fee) AS total_fee, 
        COUNT(*) AS trade_count, 
        SUM(pnl + fee) AS profit, 
        SUM((size * price) / (100000000)::numeric) AS volume 
    FROM trades
    GROUP BY user_addr
) AS subquery;
CREATE INDEX user_trades_ua ON trades(user_addr);

-- positon limit infos
CREATE TABLE position_limits (
  user_addr VARCHAR(66) NOT NULL,
  -- Hash of the non-truncated trade type
  type_hash VARCHAR(64) NOT NULL,
  -- module_address::vault::Trade<margin_type, perp_type>
  margin_type VARCHAR(512) NOT NULL,
  perp_type VARCHAR(512) NOT NULL,
  position_limit NUMERIC NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  transaction_version BIGINT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    type_hash,
    user_addr
  )
);
CREATE INDEX pl_tv_ua_mt_pt on position_limits (transaction_version, user_addr, margin_type, perp_type);

-- market activities
CREATE TABLE market_activities (
  transaction_version BIGINT NOT NULL,
  event_creation_number BIGINT NOT NULL,
  event_sequence_number BIGINT NOT NULL,
  event_index BIGINT NOT NULL,
  -- Hash of the non-truncated vault type
  type_hash VARCHAR(64) NOT NULL,
   -- module_address::vault_events::event_type<collateral_type, borrow_type>
  event_type VARCHAR(5000) NOT NULL,
  margin_type VARCHAR(5000) NOT NULL,
  perp_type VARCHAR(5000) NOT NULL,
  user_addr VARCHAR(66),
  position_limit NUMERIC,
  id NUMERIC,
  perp_price NUMERIC,
  is_long BOOLEAN,
  margin_amount NUMERIC,
  position_size NUMERIC,
  maintenance_margin NUMERIC,
  fee NUMERIC,
  pnl NUMERIC,
  caller_addr VARCHAR(66),
  take_profit_price NUMERIC,
  stop_loss_price NUMERIC,
  trigger_price NUMERIC,
  max_price_slippage NUMERIC,
  is_increase BOOLEAN,
  triggers_above BOOLEAN,
  expiration NUMERIC,
  trigger_payment_amount NUMERIC,
  next_funding_pos BOOLEAN,
  next_funding_rate NUMERIC,
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Constraints
  PRIMARY KEY (
    transaction_version,
    event_creation_number,
    event_sequence_number
  )
);
CREATE INDEX ma_mt_pt_et_sn on market_activities (margin_type, perp_type, event_type, event_sequence_number);
CREATE INDEX ma_th_et_sn on market_activities (type_hash, event_type, event_sequence_number);
