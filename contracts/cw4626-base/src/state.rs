use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub underlying_token_address: Addr,
    pub share_token_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
