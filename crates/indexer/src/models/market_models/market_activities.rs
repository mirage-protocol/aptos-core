// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::market_events::MarketEvent;

use crate::{
    schema::{
        closed_limit_orders,
        market_activities,
        open_limit_orders,
        trades,
    },
    util::{parse_timestamp, standardize_address},
    models::market_models::markets::{Market, MarketConfig},
    models::market_models::traders::{Position, PositionLimit},
    models::market_models::limit_orders::LimitOrder,
    models::mirage::{trunc_type, hash_types}
};
use aptos_api_types::{
    Event as APIEvent, Transaction as APITransaction,
    WriteSetChange, MoveType,
};

use bigdecimal::{Zero, BigDecimal};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(
    transaction_version,
    event_index,
    margin_type,
    perp_type,
))]
#[diesel(table_name = market_activities)]
pub struct MarketActivity {
    pub transaction_version: i64,
    pub event_creation_number: i64,
    pub event_sequence_number: i64,
    pub event_index: i64,
    pub event_type: String,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: Option<String>,
    pub position_limit: Option<BigDecimal>,
    pub id: Option<BigDecimal>,
    pub perp_price: Option<BigDecimal>,
    pub is_long: Option<bool>,
    pub margin_amount: Option<BigDecimal>,
    pub position_size: Option<BigDecimal>,
    pub maintenance_margin: Option<BigDecimal>,
    pub fee: Option<BigDecimal>,
    pub pnl: Option<BigDecimal>,
    pub caller_addr: Option<String>,
    pub take_profit_price: Option<BigDecimal>,
    pub stop_loss_price: Option<BigDecimal>,
    pub trigger_price: Option<BigDecimal>,
    pub max_price_slippage: Option<BigDecimal>,
    pub is_increase: Option<bool>,
    pub triggers_above: Option<bool>,
    pub trigger_payment_amount: Option<BigDecimal>,
    pub expiration: Option<BigDecimal>,
    pub next_funding_pos: Option<bool>,
    pub next_funding_rate: Option<BigDecimal>,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

/// A simplified MarketActivity (excluded common fields) to reduce code duplication
struct MarketActivityHelper {
    pub user_addr: Option<String>,
    pub position_limit: Option<BigDecimal>,
    pub id: Option<BigDecimal>,
    pub perp_price: Option<BigDecimal>,
    pub is_long: Option<bool>,
    pub margin_amount: Option<BigDecimal>,
    pub position_size: Option<BigDecimal>,
    pub maintenance_margin: Option<BigDecimal>,
    pub fee: Option<BigDecimal>,
    pub pnl: Option<BigDecimal>,
    pub caller_addr: Option<String>,
    pub take_profit_price: Option<BigDecimal>,
    pub stop_loss_price: Option<BigDecimal>,
    pub trigger_price: Option<BigDecimal>,
    pub max_price_slippage: Option<BigDecimal>,
    pub is_increase: Option<bool>,
    pub triggers_above: Option<bool>,
    pub trigger_payment_amount: Option<BigDecimal>,
    pub expiration: Option<BigDecimal>,
    pub next_funding_pos: Option<bool>,
    pub next_funding_rate: Option<BigDecimal>,
}

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(id, transaction_version))]
#[diesel(table_name = trades)]
pub struct Trade {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: String,

    pub id: BigDecimal,
    pub is_long: bool,
    pub size: BigDecimal,
    pub price: BigDecimal,
    pub fee: BigDecimal,
    pub pnl: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = open_limit_orders)]
pub struct OpenLimitOrder {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: String,

    pub id: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = closed_limit_orders)]
pub struct ClosedLimitOrder {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: String,

    pub id: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl MarketActivity {
    /// There are different objects containing different information about the market module.
    /// Events
    /// Market Resource
    /// Trader Resource
    /// LimitOrders Resource
    pub fn from_transaction(
        transaction: &APITransaction
    ) -> (
        Vec<Market>,
        Vec<MarketConfig>,
        Vec<Position>,
        Vec<Trade>,
        Vec<PositionLimit>,
        Vec<LimitOrder>,
        Vec<OpenLimitOrder>,
        Vec<ClosedLimitOrder>,
        Vec<MarketActivity>,
    ) {
        let mut markets: Vec<Market> = Vec::new();
        let mut market_configs: Vec<MarketConfig> = Vec::new();
        let mut positions: Vec<Position> = Vec::new();
        let mut trades: Vec<Trade> = Vec::new();
        let mut position_limits: Vec<PositionLimit> = Vec::new();
        let mut limit_orders: Vec<LimitOrder> = Vec::new();
        let mut open_limit_orders: Vec<OpenLimitOrder> = Vec::new();
        let mut closed_limit_orders: Vec<ClosedLimitOrder> = Vec::new();
        let mut market_activities: Vec<MarketActivity> = Vec::new();

        let (writesets, events, txn_version, txn_timestamp) = match &transaction {
            APITransaction::UserTransaction(inner) => (
                &inner.info.changes,
                &inner.events,
                inner.info.version.0 as i64,
                parse_timestamp(inner.timestamp.0, inner.info.version.0 as i64),
            ),
            _ => return Default::default(),
        };

        for wsc in writesets {
            let (maybe_market_info, maybe_position_info, maybe_limit_orders) =
                if let WriteSetChange::WriteResource(write_resource) = wsc {
                    (
                        Market::from_write_resource(write_resource, txn_version, txn_timestamp)
                            .unwrap(),
                        Position::from_write_resource(write_resource, txn_version, txn_timestamp)
                        .unwrap(),
                        LimitOrder::from_write_resource(write_resource, txn_version, txn_timestamp)
                        .unwrap(),
                    )
                } else {
                    (None, None, None)
                };

            if let Some((market, market_config)) = maybe_market_info {
                markets.push(market);
                market_configs.push(market_config);
            }
            if let Some((position, position_limit)) = maybe_position_info {
                positions.push(position);
                position_limits.push(position_limit);
            }
            if let Some(limit_order) = maybe_limit_orders {
                limit_orders.extend(limit_order);
            }
        }

        for (index, event) in events.iter().enumerate() {
            if let MoveType::Struct(inner) = &event.typ {
                if MarketEvent::is_event_supported(inner) {
                    let maybe_market_event = MarketEvent::from_event(&inner.name.to_string(), &event.data, txn_version);

                    if let Ok(market_event) = maybe_market_event {
                        let margin_type = &inner.generic_type_params[0];
                        let perp_type = &inner.generic_type_params[1];
                        let (
                            market_activity,
                            maybe_trade,
                            maybe_open_limit_order, maybe_closed_limit_order
                            ) = Self::from_parsed_event(
                                &inner.name.to_string(),
                                &margin_type.to_string(),
                                &perp_type.to_string(),
                                event,
                                &market_event,
                                txn_version,
                                txn_timestamp,
                                index as i64,
                            );
                        market_activities.push(market_activity);


                        if let Some(inner) = maybe_trade {
                            trades.push(inner);
                        }

                        if let Some(inner) = maybe_open_limit_order {
                            open_limit_orders.push(inner);
                        }

                        if let Some(inner) = maybe_closed_limit_order {
                            closed_limit_orders.push(inner);
                        }
                    };
                }
            }
        };

        (
            markets,
            market_configs,
            positions,
            trades,
            position_limits,
            limit_orders,
            open_limit_orders,
            closed_limit_orders,
            market_activities,
        )
    }

    fn from_parsed_event(
        event_type: &String,
        margin_type: &String,
        perp_type: &String,
        event: &APIEvent,
        parsed_event: &MarketEvent,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
        event_index: i64,
    ) -> (
        MarketActivity,
        Option<Trade>,
        Option<OpenLimitOrder>,
        Option<ClosedLimitOrder>,
     ) {
        let event_creation_number = event.guid.creation_number.0 as i64;
        let event_sequence_number = event.sequence_number.0 as i64;

        let mut trade = None;

        let mut open_limit_order = None;
        let mut closed_limit_order = None;

        let market_activity_helper = match parsed_event {
            MarketEvent::RegisterUserEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: Some(inner.position_limit.clone()),
                id: None,
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                pnl: None,
                fee: None,
                caller_addr: None,
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::UpdatePositionLimitEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: Some(inner.position_limit.clone()),
                id: None,
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                pnl: None,
                fee: None,
                caller_addr: None,
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::OpenPositionEvent(inner) => {
                trade = Some(Trade {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    is_long: inner.is_long,
                    size: inner.position_size.clone(),
                    fee: inner.fee.clone(),
                    pnl: BigDecimal::zero(),
                    price: inner.opening_price.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: Some(inner.opening_price.clone()),
                    is_long: Some(inner.is_long),
                    margin_amount: Some(inner.margin_amount.clone()),
                    position_size: Some(inner.position_size.clone()),
                    maintenance_margin: Some(inner.maintenance_margin.clone()),
                    fee: Some(inner.fee.clone()),
                    pnl: None,
                    caller_addr: None,
                    take_profit_price: Some(inner.take_profit_price.clone()),
                    stop_loss_price: Some(inner.stop_loss_price.clone()),
                    trigger_price: None,
                    max_price_slippage: None,
                    is_increase: None,
                    triggers_above: None,
                    expiration: None,
                    trigger_payment_amount: None,
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            },
            MarketEvent::ClosePositionEvent(inner) => {
                let pnl = if inner.winner { inner.pnl.clone() } else { -inner.pnl.clone() };
                trade = Some(Trade {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    is_long: inner.is_long,
                    size: inner.position_size.clone(),
                    price: inner.closing_price.clone(),
                    fee: inner.fee.clone(),
                    pnl: pnl.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: Some(inner.closing_price.clone()),
                    is_long: None,
                    margin_amount: None,
                    position_size: None,
                    maintenance_margin: None,
                    fee: Some(inner.fee.clone()),
                    pnl: Some(pnl),
                    caller_addr: None,
                    take_profit_price: None,
                    stop_loss_price: None,
                    trigger_price: None,
                    max_price_slippage: None,
                    is_increase: None,
                    triggers_above: None,
                    expiration: None,
                    trigger_payment_amount: None,
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            },
            MarketEvent::UpdateMarginEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: None,
                id: Some(inner.id.clone()),
                perp_price: None,
                is_long: None,
                margin_amount: Some(inner.margin_amount.clone()),
                position_size: None,
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: None,
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::UpdatePositionSizeEvent(inner) => {
                let pnl = if inner.winner { inner.pnl.clone() } else { -inner.pnl.clone() };
                trade = Some(Trade {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    is_long: inner.is_long,
                    fee: inner.fee.clone(),
                    pnl: pnl.clone(),
                    size: inner.position_size.clone() - inner.prev_position_size.clone(),
                    price: inner.new_opening_price.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: Some(inner.new_opening_price.clone()),
                    is_long: None,
                    margin_amount: None,
                    position_size: Some(inner.position_size.clone()),
                    maintenance_margin: None,
                    fee: Some(inner.fee.clone()),
                    pnl: Some(pnl.clone()),
                    caller_addr: None,
                    take_profit_price: None,
                    stop_loss_price: None,
                    trigger_price: None,
                    max_price_slippage: None,
                    is_increase: None,
                    triggers_above: None,
                    expiration: None,
                    trigger_payment_amount: None,
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            },
            MarketEvent::LiquidatePositionEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: None,
                id: Some(inner.id.clone()),
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: Some(standardize_address(&inner.liquidator_addr)),
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::UpdateTpslEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: None,
                id: Some(inner.id.clone()),
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: None,
                take_profit_price: Some(inner.take_profit_price.clone()),
                stop_loss_price: Some(inner.stop_loss_price.clone()),
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::TriggerTpslEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: None,
                id: Some(inner.id.clone()),
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: Some(standardize_address(&inner.caller_addr)),
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::PlaceLimitOrderEvent(inner) => {
                open_limit_order = Some(OpenLimitOrder {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: None,
                    is_long: Some(inner.is_long),
                    margin_amount: Some(inner.margin_amount.clone()),
                    position_size: Some(inner.position_size.clone()),
                    maintenance_margin: None,
                    fee: None,
                    pnl: None,
                    caller_addr: None,
                    take_profit_price: None,
                    stop_loss_price: None,
                    trigger_price: Some(inner.trigger_price.clone()),
                    max_price_slippage: Some(inner.max_price_slippage.clone()),
                    is_increase: Some(inner.is_increase),
                    triggers_above: Some(inner.triggers_above),
                    expiration: Some(inner.expiration.clone()),
                    trigger_payment_amount: Some(inner.trigger_payment_amount.clone()),
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            }
            MarketEvent::UpdateLimitOrderEvent(inner) => MarketActivityHelper {
                user_addr: Some(standardize_address(&inner.user_addr)),
                position_limit: None,
                id: Some(inner.id.clone()),
                perp_price: None,
                is_long: None,
                margin_amount: Some(inner.margin_amount.clone()),
                position_size: Some(inner.position_size.clone()),
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: None,
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: Some(inner.trigger_price.clone()),
                max_price_slippage: Some(inner.max_price_slippage.clone()),
                is_increase: None,
                triggers_above: Some(inner.triggers_above),
                expiration: Some(inner.expiration.clone()),
                trigger_payment_amount: Some(inner.trigger_payment_amount.clone()),
                next_funding_pos: None,
                next_funding_rate: None,
            },
            MarketEvent::CancelLimitOrderEvent(inner) => {
                closed_limit_order = Some(ClosedLimitOrder {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: None,
                    is_long: None,
                    margin_amount: None,
                    position_size: None,
                    maintenance_margin: None,
                    fee: None,
                    pnl: None,
                    caller_addr: None,
                    take_profit_price: None,
                    stop_loss_price: None,
                    trigger_price: None,
                    max_price_slippage: None,
                    is_increase: None,
                    triggers_above: None,
                    expiration: None,
                    trigger_payment_amount: None,
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            },
            MarketEvent::TriggerLimitOrderEvent(inner) => {
                closed_limit_order = Some(ClosedLimitOrder {
                    transaction_version: txn_version,
                    type_hash: hash_types(&margin_type, &perp_type),
                    margin_type: trunc_type(&margin_type),
                    perp_type: trunc_type(&perp_type),
                    user_addr: standardize_address(&inner.user_addr),
                    id: inner.id.clone(),
                    transaction_timestamp: txn_timestamp,
                });

                MarketActivityHelper {
                    user_addr: Some(standardize_address(&inner.user_addr)),
                    position_limit: None,
                    id: Some(inner.id.clone()),
                    perp_price: None,
                    is_long: None,
                    margin_amount: None,
                    position_size: None,
                    maintenance_margin: None,
                    fee: None,
                    pnl: None,
                    caller_addr: Some(standardize_address(&inner.caller_addr)),
                    take_profit_price: None,
                    stop_loss_price: None,
                    trigger_price: None,
                    max_price_slippage: None,
                    is_increase: None,
                    triggers_above: None,
                    expiration: None,
                    trigger_payment_amount: None,
                    next_funding_pos: None,
                    next_funding_rate: None,
                }
            },
            MarketEvent::UpdateFundingEvent(inner) => MarketActivityHelper {
                user_addr: None,
                position_limit: None,
                id: None,
                perp_price: None,
                is_long: None,
                margin_amount: None,
                position_size: None,
                maintenance_margin: None,
                fee: None,
                pnl: None,
                caller_addr: None,
                take_profit_price: None,
                stop_loss_price: None,
                trigger_price: None,
                max_price_slippage: None,
                is_increase: None,
                triggers_above: None,
                expiration: None,
                trigger_payment_amount: None,
                next_funding_pos: Some(inner.next_funding_pos),
                next_funding_rate: Some(inner.next_funding_rate.clone()),
            },
        };
        (
            MarketActivity {
                transaction_version: txn_version,
                event_creation_number,
                event_sequence_number,
                event_type: event_type.clone(),
                type_hash: hash_types(margin_type, perp_type),
                margin_type: trunc_type(margin_type),
                perp_type: trunc_type(perp_type),
                event_index,
                user_addr: market_activity_helper.user_addr.clone(),
                position_limit: market_activity_helper.position_limit.clone(),
                id: market_activity_helper.id.clone(),
                perp_price: market_activity_helper.perp_price.clone(),
                is_long: market_activity_helper.is_long,
                margin_amount: market_activity_helper.margin_amount.clone(),
                position_size: market_activity_helper.position_size.clone(),
                maintenance_margin: market_activity_helper.maintenance_margin.clone(),
                fee: market_activity_helper.fee.clone(),
                pnl: market_activity_helper.pnl.clone(),
                caller_addr: market_activity_helper.caller_addr.clone(),
                take_profit_price: market_activity_helper.take_profit_price.clone(),
                stop_loss_price: market_activity_helper.stop_loss_price.clone(),
                trigger_price: market_activity_helper.trigger_price.clone(),
                max_price_slippage: market_activity_helper.max_price_slippage.clone(),
                is_increase: market_activity_helper.is_increase,
                triggers_above: market_activity_helper.triggers_above,
                expiration: market_activity_helper.expiration.clone(),
                trigger_payment_amount: market_activity_helper.trigger_payment_amount.clone(),
                next_funding_pos: market_activity_helper.next_funding_pos,
                next_funding_rate: market_activity_helper.next_funding_rate.clone(),
                transaction_timestamp: txn_timestamp,
            },
            trade,
            open_limit_order,
            closed_limit_order,
        )
    }
}
