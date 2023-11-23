// Copyright © Mirage Protocol

/**
 * This file defines deserialized rebase module types.
 */

use aptos_api_types::deserialize_from_string;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use super::coin_models::coin_utils::{Coin};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rebase {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub elastic: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base: BigDecimal,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoinRebase {
    pub elastic: Coin,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub amount: BigDecimal,
}
