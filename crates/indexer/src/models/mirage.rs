// Copyright © Mirage Protocol

use crate::util::{hash_str, truncate_str};

pub const MIRAGE_ADDRESS: &str = "0x9e7f2c6a5c7de8d4160474564b7cafc072f2b7308abea1361d7efb3d51f012ac";
pub const MIRAGE_TYPE_MAX_LENGTH: usize = 512;

pub fn trunc_type(move_type: &str) -> String {
    truncate_str(move_type, MIRAGE_TYPE_MAX_LENGTH)
}

pub fn hash_types(collateral_type: &str, borrow_type: &str) -> String {
    hash_str(&format!("<{},{}>", &trunc_type(collateral_type), &trunc_type(borrow_type)))
}
