use std::{collections::HashMap, fmt::Display};

use astroport::asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

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

#[cw_serde]
pub struct TowerConfig {
    pub lp: Addr,
    pub lp_underlying_asset: AssetInfo,
    pub lp_other_asset: AssetInfo,
    pub lp_token: Addr,
    pub incentives: Vec<AssetInfo>,
    pub slippage_tolerance: Decimal,
}

pub const UNDERLYING_ASSET: Item<Addr> = Item::new("asset");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");
pub const ACCESS_CONTROL: Map<String, Addr> = Map::new("access-control");
pub const TOWER_CONFIG: Item<TowerConfig> = Item::new("tower-config");
/// Prices map in terms of the underlying asset
/// NOTE: It's an Item of a HashMap and not a Map because it needs to be read & updated completely every time
pub const ORACLE_PRICES: Item<HashMap<String, Decimal>> = Item::new("oracle-prices");
