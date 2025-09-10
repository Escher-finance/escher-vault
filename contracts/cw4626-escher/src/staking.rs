use astroport::asset::Asset;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Decimal, DepsMut, StdError, Timestamp, Uint128, WasmMsg};

use crate::{
    asset::{asset_cw20_send_or_attach_funds, query_asset_info_balance},
    helpers::validate_salt,
    state::UNDERLYING_ASSET,
    ContractError,
};

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
    Unstake {
        amount: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        recipient_ibc_channel_id: Option<u32>,
    },
}

/// Returns (`bond_msg`, `expected`)
///
/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn internal_bond(
    deps: &mut DepsMut,
    this: &Addr,
    staking_contract: &Addr,
    amount: Uint128,
    salt: String,
    slippage: Option<Decimal>,
) -> Result<(WasmMsg, Uint128), ContractError> {
    validate_salt(&salt)?;

    let EscherHubStakingLiquidity { exchange_rate, .. } = deps.querier.query_wasm_smart(
        staking_contract.clone(),
        &EscherHubQueryMsg::StakingLiquidity {},
    )?;

    let expected = amount
        .checked_div_floor(exchange_rate)
        .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    // Get the current asset balance in the vault
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset_balance = query_asset_info_balance(&deps.querier, asset_info.clone(), this)?;

    // Validate that we have enough assets to bond
    if asset_balance < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // Create the bond message for the staking contract
    let bond_msg = asset_cw20_send_or_attach_funds(
        Asset {
            info: asset_info,
            amount,
        },
        staking_contract,
        to_json_binary(&EscherHubExecuteMsg::Bond {
            slippage,
            expected,
            recipient: None,
            recipient_channel_id: None,
            salt: Some(salt),
        })?,
    )?;

    Ok((bond_msg, expected))
}
