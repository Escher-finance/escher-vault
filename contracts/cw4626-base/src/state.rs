use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const UNDERLYING_ASSET: Item<Addr> = Item::new("asset");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");

pub const SHARE: Item<Addr> = Item::new("share");
