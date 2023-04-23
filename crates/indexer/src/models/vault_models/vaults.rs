// Copyright © Mirage Protocol

use super::vault_utils::{
    hash_types,
    trunc_type,
    VaultModuleResource
};

use crate::schema::vaults;

use aptos_api_types::WriteResource;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use field_count::FieldCount;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(collateral_type, borrow_type))]
#[diesel(table_name = vaults)]
pub struct Vault {
    pub transaction_version: i64,
    pub collateral_type: String,
    pub borrow_type: String,
    pub type_hash: String,
    pub total_collateral: BigDecimal,
    pub borrow_elastic: BigDecimal,
    pub borrow_base: BigDecimal,
    pub last_fees_accrue_time: BigDecimal,
    pub fees_accrued: BigDecimal,
    pub interest_per_second: BigDecimal,
    pub collateralization_rate: BigDecimal,
    pub liquidation_multiplier: BigDecimal,
	pub borrow_fee: BigDecimal,
    pub distribution_part: BigDecimal,
    pub fee_to: String,
	pub cached_exchange_rate: BigDecimal,
	pub last_interest_update: BigDecimal,
	pub is_emergency: bool,
	pub dev_cut: BigDecimal,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

impl Vault {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<Self>> {
        match &VaultModuleResource::from_write_resource(write_resource, txn_version)? {
            Some(VaultModuleResource::VaultResource(inner)) => {
                let collateral_type = &write_resource.data.typ.generic_type_params[0].to_string();
                let borrow_type = &write_resource.data.typ.generic_type_params[1].to_string();

                Ok(Some(Self {
                    transaction_version: txn_version,
                    type_hash: hash_types(collateral_type, borrow_type),
                    collateral_type: trunc_type(collateral_type),
                    borrow_type: trunc_type(borrow_type),
                    total_collateral: inner.total_collateral.clone(),
                    borrow_elastic: inner.borrow.elastic.clone(),
                    borrow_base: inner.borrow.base.clone(),
                    last_fees_accrue_time: inner.fees.last_time.clone(),
                    fees_accrued: inner.fees.fees_earned.value.clone(),
                    interest_per_second: inner.interest_per_second.clone(),
                    collateralization_rate: inner.collateralization_rate.clone(),
                    liquidation_multiplier: inner.liquidation_multiplier.clone(),
                    borrow_fee: inner.borrow_fee.clone(),
                    distribution_part: inner.distribution_part.clone(),
                    fee_to: inner.fee_to.clone(),
                    cached_exchange_rate: inner.cached_exchange_rate.clone(),
                    last_interest_update: inner.last_interest_update.clone(),
                    is_emergency: inner.emergency,
                    dev_cut: inner.dev_cut.clone(),
                    transaction_timestamp: txn_timestamp,
                }))
            },
            _ => Ok(None)
        }
    }
}
