use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp, Uint128};

#[cw_serde]
#[derive(Default)]
pub struct EscherHubStakingLiquidity {
    pub amount: Uint128,
    pub delegated: Uint128,
    pub reward: Uint128,
    pub unclaimed_reward: Uint128,
    pub exchange_rate: Decimal,
    pub time: Timestamp,
    pub total_supply: Uint128,
    pub adjusted_supply: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum EscherHubQueryMsg {
    #[returns(EscherHubStakingLiquidity)]
    StakingLiquidity {},
}

#[cw_serde]
pub enum EscherHubExecuteMsg {
    Bond {
        slippage: Option<Decimal>,
        expected: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        salt: Option<String>,
    },
    Unbond {
        amount: Uint128,
        salt: Option<String>,
    },
}
