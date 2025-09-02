use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

// TODO: also use the query to validate that the staking addr is correct

#[cw_serde]
pub enum EscherHubExecuteMsg {
    Bond {
        slippage: Option<Decimal>,
        expected: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        salt: Option<String>,
    },
}
