// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]

/**
 * This file defines resources deserialized vault module.
 */

use super::rebase::Rebase;

use crate::{
    util::{standardize_address, hash_str, truncate_str},
    models::coin_models::coin_utils::{Coin},
    models::move_resources::MoveResource,
};

use aptos_api_types::{deserialize_from_string, WriteResource, MoveStructTag};

use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

pub const MIRAGE_ADDRESS: &str = "0x701cdfb5e87de07beacc835c2bcf03428ae124b869e601f23e4e59ab645bf699";
pub const MIRAGE_TYPE_MAX_LENGTH: usize = 512;

pub fn trunc_type(move_type: &str) -> String {
    truncate_str(move_type, MIRAGE_TYPE_MAX_LENGTH)
}

pub fn hash_types(collateral_type: &str, borrow_type: &str) -> String {
    hash_str(&format!("<{},{}>", &trunc_type(collateral_type), &trunc_type(borrow_type)))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeeAccrureInfo {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub last_time: BigDecimal,
    pub fees_earned: Coin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VaultResource {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_collateral: BigDecimal,
    pub borrow: Rebase,
    pub fees: FeeAccrureInfo,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub interest_per_second: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub collateralization_rate: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub liquidation_multiplier: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub borrow_fee: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub distribution_part: BigDecimal,
    pub fee_to: String,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub cached_exchange_rate: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub last_interest_update: BigDecimal,
	pub emergency: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
	pub dev_cut: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfoResource {
    pub user_collateral: Coin,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub user_borrow_part: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VaultModuleResource {
    UserInfoResource(UserInfoResource),
    VaultResource(VaultResource),
}

impl VaultModuleResource {
    pub fn is_resource_supported(move_type: &MoveStructTag) -> bool {
        standardize_address(&move_type.address.to_string()) == MIRAGE_ADDRESS
            && move_type.module.to_string() == "vault"
            && (move_type.name.to_string() == "UserInfo"
              || move_type.name.to_string() == "Vault")
            && move_type.generic_type_params.len() == 2
    }

    pub fn from_resource(
        resource_name: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<VaultModuleResource> {
        match resource_name {
            "UserInfo" => serde_json::from_value(data.clone())
                .map(|inner| Some(VaultModuleResource::UserInfoResource(inner))),
            "Vault" => serde_json::from_value(data.clone())
                .map(|inner: VaultResource| Some(VaultModuleResource::VaultResource(inner))),
            _ => Ok(None)
        }
        .context(format!(
            "version {} failed! failed to parse vault resource {}, data {:?}",
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
    ) -> Result<Option<VaultModuleResource>> {
        if !VaultModuleResource::is_resource_supported(&write_resource.data.typ) {
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
