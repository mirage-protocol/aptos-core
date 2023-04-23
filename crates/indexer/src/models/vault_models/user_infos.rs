// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::vault_utils::{
    hash_types,
    trunc_type,
    VaultModuleResource
};

use crate::{
    schema::user_infos,
    util::standardize_address,
};

use aptos_api_types::WriteResource;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(user_address, collateral_type, borrow_type))]
#[diesel(table_name = user_infos)]
pub struct UserInfo {
    pub transaction_version: i64,
    pub collateral_type: String,
    pub borrow_type: String,
    pub type_hash: String,
    pub user_address: String,
    pub user_collateral: BigDecimal,
    pub user_borrow_part: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl UserInfo {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<Self>> {
        match &VaultModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(VaultModuleResource::UserInfoResource(inner)) => {
                let user_address = write_resource.address.to_string();
                let collateral_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let borrow_type = &write_resource.data.typ.generic_type_params[1].to_string();

                Ok(Some(Self {
                    transaction_version: txn_version,
                    user_address: standardize_address(&user_address),
                    type_hash: hash_types(&collateral_type, &borrow_type),
                    collateral_type: trunc_type(&collateral_type),
                    borrow_type: trunc_type(&borrow_type),
                    user_collateral: inner.user_collateral.value.clone(),
                    user_borrow_part: inner.user_borrow_part.clone(),
                    transaction_timestamp: txn_timestamp,
                }))
            },
            _ => Ok(None)
        }
    }
}
