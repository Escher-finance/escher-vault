use cosmwasm_std::Addr;
use cw4626::WithdrawalShareAllowanceResponse;
use cw_storage_plus::{Item, Map};

pub const ASSET: Item<Addr> = Item::new("asset");
pub const SHARE: Item<Addr> = Item::new("share");

pub const WITHDRAWAL_SHARE_ALLOWANCES: Map<(&Addr, &Addr), WithdrawalShareAllowanceResponse> =
    Map::new("allowance");
