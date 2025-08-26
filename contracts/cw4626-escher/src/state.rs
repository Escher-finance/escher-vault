use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const UNDERLYING_ASSET: Item<Addr> = Item::new("asset");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");

#[cw_serde]
pub enum AccessControlRole {
    Manager {},
    Oracle {},
}

impl Display for AccessControlRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Manager {} => "manager",
                Self::Oracle {} => "oracle",
            }
        )
    }
}

impl AccessControlRole {
    pub fn key(&self) -> String {
        self.to_string()
    }
}

pub const ACCESS_CONTROL: Map<String, Addr> = Map::new("access-control");
