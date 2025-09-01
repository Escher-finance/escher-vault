use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub enum TokenType {
    Cw20 { address: Addr },
    Native { denom: String },
}

pub const TOKEN_TYPE: Item<TokenType> = Item::new("token_type");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");

// Legacy support - keep for backward compatibility
pub const UNDERLYING_ASSET: Item<Addr> = Item::new("asset");

// Staking contract configuration
pub const STAKING_CONTRACT: Item<Addr> = Item::new("staking_contract");
