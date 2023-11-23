-- This file should undo anything in `up.sql`

DROP VIEW IF EXISTS user_trades;

DROP TABLE IF EXISTS closed_limit_orders;
DROP TABLE IF EXISTS limit_orders;
DROP TABLE IF EXISTS market_activities;
DROP TABLE IF EXISTS market_configs;
DROP TABLE IF EXISTS markets;
DROP TABLE IF EXISTS open_limit_orders;
DROP TABLE IF EXISTS position_limits;
DROP TABLE IF EXISTS positions;
DROP TABLE IF EXISTS trades;
