use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub asset_address: Addr,
    pub share_address: Addr,
    pub asset_decimals: u8,
    pub share_decimals: u8,
}

pub const CONFIG: Item<Config> = Item::new("config");
