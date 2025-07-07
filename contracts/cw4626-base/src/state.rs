use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ASSET: Item<Addr> = Item::new("asset");
pub const SHARE: Item<Addr> = Item::new("share");
