// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::market_utils::MarketModuleResource;

use crate::{
    schema::{market_configs, markets},
    models::mirage::{hash_types, trunc_type}
};

use aptos_api_types::WriteResource;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, type_hash))]
#[diesel(table_name = market_configs)]
pub struct MarketConfig {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,

    pub max_taker_fee: BigDecimal,
    pub min_taker_fee: BigDecimal,
    pub max_maker_fee: BigDecimal,
    pub min_maker_fee: BigDecimal,
    pub liquidation_fee: BigDecimal,

    pub min_funding_rate: BigDecimal,
    pub max_funding_rate: BigDecimal,
    pub pool_funding_discount: BigDecimal,
    pub funding_interval: BigDecimal,

    pub max_oi: BigDecimal,
    pub max_oi_imbalance: BigDecimal,

    pub max_leverage: BigDecimal,
    pub base_maintenance_margin: BigDecimal,

    pub base_position_limit: BigDecimal,
    pub max_position_limit: BigDecimal,

    pub min_order_size: BigDecimal,

    pub transaction_timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, type_hash))]
#[diesel(table_name = markets)]
pub struct Market {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,

    pub long_margin_base: BigDecimal,
    pub long_margin_elastic: BigDecimal,
    pub short_margin_base: BigDecimal,
    pub short_margin_elastic: BigDecimal,

    pub long_oi: BigDecimal,
    pub short_oi: BigDecimal,

    pub next_funding_rate: BigDecimal,
    pub next_funding_pos: bool,
    pub last_funding_round: BigDecimal,

    pub is_long_close_only: bool,
    pub is_short_close_only: bool,

    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl Market {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<(Market, MarketConfig)>> {
        match &MarketModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(MarketModuleResource::MarketResource(inner)) => {
                let margin_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let perp_type = &write_resource.data.typ.generic_type_params[1].to_string();

                let config = &inner.config;

                return Ok(Some(
                    (Market {
                        transaction_version: txn_version,
                        type_hash: hash_types(&margin_type, &perp_type),
                        margin_type: trunc_type(&margin_type),
                        perp_type: trunc_type(&perp_type),
                        long_margin_base: inner.long_margin.base.clone(),
                        long_margin_elastic: inner.long_margin.elastic.value.clone(),
                        short_margin_base: inner.short_margin.base.clone(),
                        short_margin_elastic: inner.short_margin.elastic.value.clone(),
                        long_oi: inner.long_oi.clone(),
                        short_oi: inner.short_oi.clone(),
                        next_funding_rate: inner.next_funding_rate.clone(),
                        next_funding_pos: inner.next_funding_pos,
                        last_funding_round: inner.last_funding_round.clone(),
                        is_long_close_only: inner.is_long_close_only,
                        is_short_close_only: inner.is_short_close_only,
                        transaction_timestamp: txn_timestamp,
                    },
                    MarketConfig {
                        transaction_version: txn_version,
                        type_hash: hash_types(&margin_type, &perp_type),
                        margin_type: trunc_type(&margin_type),
                        perp_type: trunc_type(&perp_type),
                        max_taker_fee: config.fees.max_taker_fee.clone(),
                        min_taker_fee: config.fees.min_taker_fee.clone(),
                        max_maker_fee: config.fees.max_maker_fee.clone(),
                        min_maker_fee: config.fees.min_maker_fee.clone(),
                        liquidation_fee: config.fees.liquidation_fee.clone(),
                        min_funding_rate: config.funding.min_funding_rate.clone(),
                        max_funding_rate: config.funding.max_funding_rate.clone(),
                        pool_funding_discount: config.funding.pool_funding_discount.clone(),
                        funding_interval: config.funding.funding_interval.clone(),
                        max_oi: config.max_oi.clone(),
                        max_oi_imbalance: config.max_oi_imbalance.clone(),
                        max_leverage: config.max_leverage.clone(),
                        base_maintenance_margin: config.base_maintenance_margin.clone(),
                        base_position_limit: config.base_position_limit.clone(),
                        max_position_limit: config.max_position_limit.clone(),
                        min_order_size: config.min_order_size.clone(),
                        transaction_timestamp: txn_timestamp,
                    })
                ))
            },
            _ => Ok(None)
        }
    }
}
