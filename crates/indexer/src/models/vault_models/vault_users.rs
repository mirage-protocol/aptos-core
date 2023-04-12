// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::vault_utils::VaultModuleResource;

use crate::models::mirage::{hash_types, trunc_type};

use crate::{
    schema::vault_users,
    util::standardize_address,
};

use aptos_api_types::WriteResource;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(user_addr, collateral_type, borrow_type))]
#[diesel(table_name = vault_users)]
pub struct VaultUser {
    pub transaction_version: i64,
    pub collateral_type: String,
    pub borrow_type: String,
    pub type_hash: String,
    pub user_addr: String,

    pub collateral: BigDecimal,
    pub borrow_part: BigDecimal,

    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl VaultUser {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<Self>> {
        match &VaultModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(VaultModuleResource::VaultUserResource(inner)) => {
                let user_addr = write_resource.address.to_string();

                let collateral_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let borrow_type = &write_resource.data.typ.generic_type_params[1].to_string();

                Ok(Some(Self {
                    transaction_version: txn_version,
                    user_addr: standardize_address(&user_addr),
                    type_hash: hash_types(&collateral_type, &borrow_type),
                    collateral_type: trunc_type(&collateral_type),
                    borrow_type: trunc_type(&borrow_type),
                    collateral: inner.collateral.value.clone(),
                    borrow_part: inner.borrow_part.amount.clone(),
                    transaction_timestamp: txn_timestamp,
                }))
            },
            _ => Ok(None)
        }
    }
}
