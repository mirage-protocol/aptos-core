// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod coin_processor;
pub mod default_processor;
pub mod stake_processor;
pub mod token_processor;
pub mod mirage_processor;

use self::{
    coin_processor::NAME as COIN_PROCESSOR_NAME, default_processor::NAME as DEFAULT_PROCESSOR_NAME,
    stake_processor::NAME as STAKE_PROCESSOR_NAME, token_processor::NAME as TOKEN_PROCESSOR_NAME,
    mirage_processor::NAME as VAULT_PROCESSOR_NAME,
};

pub enum Processor {
    CoinProcessor,
    DefaultProcessor,
    TokenProcessor,
    StakeProcessor,
    MirageProcessor,
}

impl Processor {
    pub fn from_string(input_str: &String) -> Self {
        match input_str.as_str() {
            DEFAULT_PROCESSOR_NAME => Self::DefaultProcessor,
            TOKEN_PROCESSOR_NAME => Self::TokenProcessor,
            COIN_PROCESSOR_NAME => Self::CoinProcessor,
            STAKE_PROCESSOR_NAME => Self::StakeProcessor,
            VAULT_PROCESSOR_NAME => Self::MirageProcessor,
            _ => panic!("Processor unsupported {}", input_str),
        }
    }
}
