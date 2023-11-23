// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]

/**
 * This file defines deserialized market_events module types as defined in mirage protocol module.
 */

use crate::models::mirage::MIRAGE_ADDRESS;

use crate::util::standardize_address;

use aptos_api_types::{deserialize_from_string, MoveStructTag};

use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterUserEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_limit: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePositionLimitEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_limit: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenPositionEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub opening_price: BigDecimal,
    pub is_long: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub margin_amount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub maintenance_margin: BigDecimal,
   #[serde(deserialize_with = "deserialize_from_string")]
    pub take_profit_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub stop_loss_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_payment_amount: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClosePositionEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    pub is_long: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub closing_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fee: BigDecimal,
    pub winner: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pnl: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct UpdateMarginEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub margin_amount: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePositionSizeEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    pub is_long: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub prev_position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub new_opening_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fee: BigDecimal,
    pub winner: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pnl: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiquidatePositionEvent {
    pub liquidator_addr: String,
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateTpslEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub take_profit_price: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub stop_loss_price: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerTpslEvent {
    pub user_addr: String,
    pub caller_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaceLimitOrderEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    pub is_long: bool,
    pub is_increase: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub margin_amount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_price: BigDecimal,
    pub triggers_above: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_payment_amount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_price_slippage: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub expiration: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateLimitOrderEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub position_size: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub margin_amount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_price: BigDecimal,
    pub triggers_above: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub trigger_payment_amount: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_price_slippage: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub expiration: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelLimitOrderEvent {
    pub user_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerLimitOrderEvent {
    pub user_addr: String,
    pub caller_addr: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub id: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateFundingEvent {
    pub next_funding_pos: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub next_funding_rate: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MarketEvent {
    RegisterUserEvent(RegisterUserEvent),
    UpdatePositionLimitEvent(UpdatePositionLimitEvent),
    OpenPositionEvent(OpenPositionEvent),
    ClosePositionEvent(ClosePositionEvent),
    UpdateMarginEvent(UpdateMarginEvent),
    UpdatePositionSizeEvent(UpdatePositionSizeEvent),
    LiquidatePositionEvent(LiquidatePositionEvent),
    UpdateTpslEvent(UpdateTpslEvent),
    TriggerTpslEvent(TriggerTpslEvent),
    PlaceLimitOrderEvent(PlaceLimitOrderEvent),
    UpdateLimitOrderEvent(UpdateLimitOrderEvent),
    CancelLimitOrderEvent(CancelLimitOrderEvent),
    TriggerLimitOrderEvent(TriggerLimitOrderEvent),
    UpdateFundingEvent(UpdateFundingEvent),
}

impl MarketEvent {
    pub fn is_event_supported(move_type: &MoveStructTag) -> bool {
        standardize_address(&move_type.address.to_string()) == MIRAGE_ADDRESS
            && move_type.module.to_string() == "market"
            && move_type.generic_type_params.len() == 2
    }

    pub fn from_event(
        event_name: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<Self> {
        match event_name {
            "RegisterUserEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::RegisterUserEvent(inner))),
            "UpdatePositionLimitEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdatePositionLimitEvent(inner))),
            "OpenPositionEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::OpenPositionEvent(inner))),
            "ClosePositionEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::ClosePositionEvent(inner))),
            "UpdateMarginEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdateMarginEvent(inner))),
            "UpdatePositionSizeEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdatePositionSizeEvent(inner))),
            "LiquidatePositionEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::LiquidatePositionEvent(inner))),
            "UpdateTpslEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdateTpslEvent(inner))),
            "TriggerTpslEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::TriggerTpslEvent(inner))),
            "PlaceLimitOrderEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::PlaceLimitOrderEvent(inner))),
            "UpdateLimitOrderEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdateLimitOrderEvent(inner))),
            "CancelLimitOrderEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::CancelLimitOrderEvent(inner))),
            "TriggerLimitOrderEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::TriggerLimitOrderEvent(inner))),
            "UpdateFundingEvent" => serde_json::from_value(data.clone())
                .map(|inner| Some(MarketEvent::UpdateFundingEvent(inner))),
             _ => Ok(None),
        }
        .context(format!(
            "version {} failed! failed to parse event {}, data {:?}",
            txn_version, event_name, data
        ))?
        .context(format!(
            "Event unsupported! Call is_event_supported first. version {} event {}",
            txn_version, event_name
        ))
    }
}
