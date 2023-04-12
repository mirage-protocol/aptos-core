// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]

/**
 * This file defines resources deserialized market module.
 */

use crate::{
    util::standardize_address,
    models::{
        coin_models::coin_utils::Coin,
        move_resources::MoveResource,
        rebase::{CoinRebase, Base},
        mirage::MIRAGE_ADDRESS
    }
};

use aptos_api_types::{deserialize_from_string, WriteResource, MoveStructTag};

use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeeInfo {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_taker_fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub min_taker_fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_maker_fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub min_maker_fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub liquidation_fee: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundingInfo {
    #[serde(deserialize_with = "deserialize_from_string")]
	pub min_funding_rate: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_funding_rate: BigDecimal,
	pub pool_funding_discount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub funding_interval: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketConfigStruct {
    pub fees: FeeInfo,
    pub funding: FundingInfo,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_oi: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_oi_imbalance: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_leverage: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_maintenance_margin: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_position_limit: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_position_limit: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub min_order_size: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketResource {
    pub long_margin: CoinRebase,
    pub short_margin: CoinRebase,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub long_oi: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub short_oi: BigDecimal,


    #[serde(deserialize_with = "deserialize_from_string")]
	pub next_funding_rate: BigDecimal,
    pub next_funding_pos: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub last_funding_round: BigDecimal,

	pub is_long_close_only: bool,
	pub is_short_close_only: bool,

    pub config: MarketConfigStruct,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub opening_price: BigDecimal,
    pub is_long: bool,
    pub margin_part: Base,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub maintenance_margin: BigDecimal,

    pub tpsl: Tpsl,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tpsl {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub take_profit_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub stop_loss_price: BigDecimal,
    pub trigger_payment: Coin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraderResource {
    pub position: Position,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_limit: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LimitOrder {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,

    pub is_long: bool,
    pub is_increase: bool,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    pub margin: Coin,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_price: BigDecimal,
    pub triggers_above: bool,
    pub trigger_payment: Coin,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_price_slippage: BigDecimal,

    #[serde(deserialize_with = "deserialize_from_string")]
    pub expiration: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LimitOrdersResource {
    pub orders: Vec<LimitOrder>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MarketModuleResource {
    TraderResource(TraderResource),
    MarketResource(MarketResource),
    LimitOrdersResource(LimitOrdersResource)
}

impl MarketModuleResource {
    pub fn is_resource_supported(move_type: &MoveStructTag) -> bool {
        standardize_address(&move_type.address.to_string()) == MIRAGE_ADDRESS
            && move_type.module.to_string() == "market"
            && (move_type.name.to_string() == "Market"
              || move_type.name.to_string() == "Trader"
              || move_type.name.to_string() == "LimitOrders")
            && move_type.generic_type_params.len() == 2
    }

    pub fn from_resource(
        resource_name: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<MarketModuleResource> {
        match resource_name {
            "Market" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketModuleResource::MarketResource(inner))),
            "Trader" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketModuleResource::TraderResource(inner))),
            "LimitOrders" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketModuleResource::LimitOrdersResource(inner))),
             _ => Ok(None)
        }
        .context(format!(
            "version {} failed! failed to parse market resource {}, data {:?}",
            txn_version, &resource_name, data
        ))?
        .context(format!(
            "Resource unsupported! Call is_resource_supported first. version {} type {}",
            txn_version, &resource_name
        ))

    }

    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
    ) -> Result<Option<MarketModuleResource>> {
        if !MarketModuleResource::is_resource_supported(&write_resource.data.typ) {
            return Ok(None);
        }

        let resource = MoveResource::from_write_resource(
            write_resource,
            0, // Placeholder, this isn't used anyway
            txn_version,
            0, // Placeholder, this isn't used anyway
        );
        Ok(Some(Self::from_resource(
            &write_resource.data.typ.name.to_string(),
            resource.data.as_ref().unwrap(),
            txn_version,
        )?))
    }
}
