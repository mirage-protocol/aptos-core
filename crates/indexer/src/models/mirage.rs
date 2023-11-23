// Copyright © Mirage Protocol

use crate::util::{hash_str, truncate_str};

pub const MIRAGE_ADDRESS: &str = "0x2fcf786835005f86fecba4f394f306e5444658f391bcaf301608ed78c8d64c65";
pub const MIRAGE_TYPE_MAX_LENGTH: usize = 512;

pub fn trunc_type(move_type: &str) -> String {
    truncate_str(move_type, MIRAGE_TYPE_MAX_LENGTH)
}

pub fn hash_types(collateral_type: &str, borrow_type: &str) -> String {
    hash_str(&format!("<{},{}>", &trunc_type(collateral_type), &trunc_type(borrow_type)))
}
