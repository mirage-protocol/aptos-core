// Copyright © Mirage Protocol

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::{
    vault_users::VaultUser,
    vault_events::VaultEvent,
    vaults::Vault,
};
use crate::{
    schema::vault_activities,
    util::{parse_timestamp, standardize_address},
    models::mirage::{trunc_type, hash_types}
};
use aptos_api_types::{
    Event as APIEvent, Transaction as APITransaction,
    WriteSetChange, MoveType,
};

use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(
    transaction_version,
    event_index,
    collateral_type,
    borrow_type,
))]
#[diesel(table_name = vault_activities)]
pub struct VaultActivity {
    pub transaction_version: i64,
    pub event_creation_number: i64,
    pub event_sequence_number: i64,
    pub event_index: i64,
    pub event_type: String,
    pub type_hash: String,
    pub collateral_type: String,
    pub borrow_type: String,
    pub collateral_amount: Option<BigDecimal>,
    pub borrow_amount: Option<BigDecimal>,
    pub user_addr: Option<String>,
    pub withdraw_addr: Option<String>,
    pub liquidator_addr: Option<String>,
    pub accrued_amount: Option<BigDecimal>,
    pub rate: Option<BigDecimal>,
    pub fees_earned: Option<BigDecimal>,
    pub old_interest_per_second: Option<BigDecimal>,
    pub new_interest_per_second: Option<BigDecimal>,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

/// A simplified VaultActivity (excluded common fields) to reduce code duplication
struct VaultActivityHelper {
    pub collateral_amount: Option<BigDecimal>,
    pub borrow_amount: Option<BigDecimal>,
    pub user_addr: Option<String>,
    pub withdraw_addr: Option<String>,
    pub liquidator_addr: Option<String>,
    pub accrued_amount: Option<BigDecimal>,
    pub rate: Option<BigDecimal>,
    pub fees_earned: Option<BigDecimal>,
    pub old_interest_per_second: Option<BigDecimal>,
    pub new_interest_per_second: Option<BigDecimal>,
}

impl VaultActivity {
    /// There are different objects containing different information about the vault module.
    /// Events
    /// VaultUser Resource
    /// Vault Resource
    pub fn from_transaction(
        transaction: &APITransaction
    ) -> (
        Vec<Vault>,
        Vec<VaultUser>,
        Vec<Self>,
    ) {
        let mut vault_users: Vec<VaultUser> = Vec::new();
        let mut vaults: Vec<Vault> = Vec::new();
        let mut vault_activities: Vec<VaultActivity> = Vec::new();

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
            let (maybe_vault, maybe_vault_user) =
                if let WriteSetChange::WriteResource(write_resource) = wsc {
                    (
                        Vault::from_write_resource(write_resource, txn_version, txn_timestamp)
                            .unwrap(),
                        VaultUser::from_write_resource(write_resource, txn_version, txn_timestamp)
                        .unwrap(),
                    )
                } else {
                    (None, None)
                };

            if let Some(inner) = maybe_vault {
                vaults.push(inner);
            }
            if let Some(inner) = maybe_vault_user {
                vault_users.push(inner);
            }
        };

        for (index, event) in events.iter().enumerate() {
            if let MoveType::Struct(inner) = &event.typ {
                if VaultEvent::is_event_supported(inner) {
                    let maybe_vault_event = VaultEvent::from_event(&inner.name.to_string(), &event.data, txn_version);

                    if let Ok(vault_event) = maybe_vault_event {
                        let collateral_type = &inner.generic_type_params[0];
                        let borrow_type = &inner.generic_type_params[1];

                        vault_activities.push(Self::from_parsed_event(
                            &inner.name.to_string(),
                            &collateral_type.to_string(),
                            &borrow_type.to_string(),
                            event,
                            &vault_event,
                            txn_version,
                            txn_timestamp,
                            index as i64,
                        ));
                    };
                }
            }
        };
        (
            vaults,
            vault_users,
            vault_activities,
        )
    }

    fn from_parsed_event(
        event_type: &String,
        collateral_type: &String,
        borrow_type: &String,
        event: &APIEvent,
        parsed_event: &VaultEvent,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
        event_index: i64,
    ) -> Self {
        let event_creation_number = event.guid.creation_number.0 as i64;
        let event_sequence_number = event.sequence_number.0 as i64;

        let vault_activity_helper = match parsed_event {
            VaultEvent::ExchangeRateEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: None,
                user_addr: None,
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: Some(inner.rate.clone()),
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::AccrueFeesEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: None,
                user_addr: None,
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: Some(inner.accrued_amount.clone()),
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::RegisterUserEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: None,
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::AddCollateralEvent(inner) => VaultActivityHelper {
                collateral_amount: Some(inner.collateral_amount.clone()),
                borrow_amount: None,
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::RemoveCollateralEvent(inner) => VaultActivityHelper {
                collateral_amount: Some(inner.collateral_amount.clone()),
                borrow_amount: None,
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::BorrowEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: Some(inner.borrow_amount.clone()),
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::RepayEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: Some(inner.borrow_amount.clone()),
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::LiquidationEvent(inner) => VaultActivityHelper {
                collateral_amount: Some(inner.collateral_amount.clone()),
                borrow_amount: Some(inner.borrow_amount.clone()),
                user_addr: Some(standardize_address(&inner.user_addr)),
                withdraw_addr: None,
                liquidator_addr: Some(standardize_address(&inner.liquidator_addr)),
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: None,
                new_interest_per_second: None
            },
            VaultEvent::InterestRateChangeEvent(inner) => VaultActivityHelper {
                collateral_amount: None,
                borrow_amount: None,
                user_addr: None,
                withdraw_addr: None,
                liquidator_addr: None,
                accrued_amount: None,
                rate: None,
                fees_earned: None,
                old_interest_per_second: Some(inner.old_interest_per_second.clone()),
                new_interest_per_second: Some(inner.new_interest_per_second.clone()),
            },
        };

        Self {
            transaction_version: txn_version,
            event_creation_number,
            event_sequence_number,
            event_type: event_type.clone(),
            type_hash: hash_types(collateral_type, borrow_type),
            collateral_type: trunc_type(collateral_type),
            borrow_type: trunc_type(borrow_type),
            event_index,
            collateral_amount: vault_activity_helper.collateral_amount,
            borrow_amount: vault_activity_helper.borrow_amount,
            user_addr: vault_activity_helper.user_addr,
            withdraw_addr: vault_activity_helper.withdraw_addr,
            liquidator_addr: vault_activity_helper.liquidator_addr,
            accrued_amount: vault_activity_helper.accrued_amount,
            rate: vault_activity_helper.rate,
            fees_earned: vault_activity_helper.fees_earned,
            old_interest_per_second: vault_activity_helper.old_interest_per_second,
            new_interest_per_second: vault_activity_helper.new_interest_per_second,
            transaction_timestamp: txn_timestamp,
        }
    }
}
