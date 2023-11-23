// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::market_utils::MarketModuleResource;

use crate::{
    schema::limit_orders,
    util::standardize_address,
    models::mirage::{hash_types, trunc_type}
};

use aptos_api_types::WriteResource;

use bigdecimal::BigDecimal;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, id))]
#[diesel(table_name = limit_orders)]
pub struct LimitOrder {
    pub transaction_version: i64,
    pub margin_type: String,
    pub perp_type: String,
    pub type_hash: String,
    pub user_addr: String,

    pub id: BigDecimal,
    pub order_index: BigDecimal,

    pub is_long: bool,
    pub is_increase: bool,

    pub position_size: BigDecimal,
    pub margin: BigDecimal,

    pub trigger_price: BigDecimal,
    pub triggers_above: bool,
    pub trigger_payment: BigDecimal,

    pub max_price_slippage: BigDecimal,

    pub expiration: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl LimitOrder {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<Vec<LimitOrder>>> {
        match &MarketModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(MarketModuleResource::LimitOrdersResource(inner)) => {
                let user_addr = write_resource.address.to_string();
                let margin_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let perp_type = &write_resource.data.typ.generic_type_params[1].to_string();

                let mut result = Vec::new();
                result.reserve_exact(inner.orders.len());

                for index in 0..inner.orders.len() {
                    let order = &inner.orders[index];
                    result.push(LimitOrder {
                        transaction_version: txn_version,
                        user_addr: standardize_address(&user_addr),
                        type_hash: hash_types(&margin_type, &perp_type),
                        margin_type: trunc_type(&margin_type),
                        perp_type: trunc_type(&perp_type),
                        id: order.id.clone(),
                        order_index: BigDecimal::from_str(&index.to_string()).unwrap(),
                        is_long: order.is_long,
                        is_increase: order.is_increase,
                        position_size: order.position_size.clone(),
                        margin: order.margin.value.clone(),
                        trigger_price: order.trigger_price.clone(),
                        triggers_above: order.triggers_above,
                        trigger_payment: order.trigger_payment.value.clone(),
                        max_price_slippage: order.max_price_slippage.clone(),
                        expiration: order.expiration.clone(),
                        transaction_timestamp: txn_timestamp,
                    })
                }
                Ok(Some(result))
            },
            _ => Ok(None)
        }
    }
}
