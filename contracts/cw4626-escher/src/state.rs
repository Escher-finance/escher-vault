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
    pub tower_incentives: Addr,
    pub lp: Addr,
    pub lp_underlying_asset: AssetInfo,
    pub lp_other_asset: AssetInfo,
    pub is_underlying_first_lp_asset: bool,
    pub lp_token: Addr,
    pub lp_incentives: Vec<AssetInfo>,
    pub slippage_tolerance: Decimal,
}

pub type PricesMap = HashMap<String, Decimal>;

pub const UNDERLYING_ASSET: Item<AssetInfo> = Item::new("asset");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");
pub const ACCESS_CONTROL: Map<String, Vec<Addr>> = Map::new("access-control");
pub const TOWER_CONFIG: Item<TowerConfig> = Item::new("tower-config");
/// Prices map in terms of the underlying asset
/// NOTE: It's an Item of a HashMap and not a Map because it needs to be read & updated completely every time
pub const ORACLE_PRICES: Item<PricesMap> = Item::new("oracle-prices");
// Staking contract configuration
pub const STAKING_CONTRACT: Item<Addr> = Item::new("staking_contract");
