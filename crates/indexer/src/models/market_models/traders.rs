// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::market_utils::MarketModuleResource;

use crate::{
    schema::{positions, position_limits},
    util::standardize_address,
    models::mirage::{hash_types, trunc_type},
};

use aptos_api_types::WriteResource;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, type_hash, user_addr))]
#[diesel(table_name = position_limits)]
pub struct PositionLimit {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: String,

    pub position_limit: BigDecimal,

    pub transaction_timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, id))]
#[diesel(table_name = positions)]
pub struct Position {
    pub transaction_version: i64,
    pub type_hash: String,
    pub margin_type: String,
    pub perp_type: String,
    pub user_addr: String,

    pub id: BigDecimal,
    pub opening_price: BigDecimal,
    pub is_long: bool,
    pub margin_part: BigDecimal,
    pub position_size: BigDecimal,

    pub maintenance_margin: BigDecimal,

    pub take_profit_price: BigDecimal,
    pub stop_loss_price: BigDecimal,
    pub trigger_payment: BigDecimal,

    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl Position {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<(Position, PositionLimit)>> {
        match &MarketModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(MarketModuleResource::TraderResource(inner)) => {
                let user_addr = write_resource.address.to_string();

                let margin_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let perp_type = &write_resource.data.typ.generic_type_params[1].to_string();

                let position_limit = &inner.position_limit;
                let position = &inner.position;

                return Ok(Some(
                    (Position {
                        transaction_version: txn_version,
                        type_hash: hash_types(&margin_type, &perp_type),
                        margin_type: trunc_type(&margin_type),
                        perp_type: trunc_type(&perp_type),
                        user_addr: standardize_address(&user_addr),
                        id: position.id.clone(),
                        opening_price: position.opening_price.clone(),
                        is_long: position.is_long,
                        margin_part: position.margin_part.amount.clone(),
                        position_size: position.position_size.clone(),
                        maintenance_margin: position.maintenance_margin.clone(),
                        take_profit_price: position.tpsl.take_profit_price.clone(),
                        stop_loss_price: position.tpsl.stop_loss_price.clone(),
                        trigger_payment: position.tpsl.trigger_payment.value.clone(),
                        transaction_timestamp: txn_timestamp,
                    },
                    PositionLimit {
                        transaction_version: txn_version,
                        type_hash: hash_types(&margin_type, &perp_type),
                        margin_type: trunc_type(&margin_type),
                        perp_type: trunc_type(&perp_type),
                        user_addr: standardize_address(&user_addr),
                        position_limit: position_limit.clone(),
                        transaction_timestamp: txn_timestamp,
                    })
                ))
            },
            _ => Ok(None)
        }
    }
}
