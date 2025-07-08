use cosmwasm_std::{
    Addr, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdError, StdResult, Storage,
    Uint128,
};
use cw4626::cw20;

use crate::{
    state::{SHARE, UNDERLYING_ASSET},
    ContractError,
};

pub fn validate_cw20(
    querier: &QuerierWrapper,
    token_address: &Addr,
) -> Result<cw20::TokenInfoResponse, ContractError> {
    querier
        .query_wasm_smart::<cw20::TokenInfoResponse>(
            token_address,
            &cw20::Cw20QueryMsg::TokenInfo {},
        )
        .map_err(|_| ContractError::InvalidCw20 {
            addr: token_address.to_string(),
        })
}

pub fn validate_share_connected(storage: &dyn Storage) -> Result<(), ContractError> {
    if SHARE.may_load(storage)?.is_none() {
        return Err(ContractError::ShareTokenNotConnected {});
    }
    Ok(())
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    token: &Addr,
    user: &Addr,
) -> Result<Uint128, StdError> {
    let cw20::BalanceResponse { balance } = querier.query_wasm_smart(
        token,
        &cw20::Cw20QueryMsg::Balance {
            address: user.to_string(),
        },
    )?;
    Ok(balance)
}

#[derive(Debug)]
pub struct Tokens {
    pub share: Addr,
    pub asset: Addr,
    pub total_shares: Uint128,
    pub total_assets: Uint128,
}

pub fn get_tokens(this: &Addr, deps: &Deps) -> StdResult<Tokens> {
    let share = this.clone();
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let total_shares = cw20_base::state::TOKEN_INFO
        .load(deps.storage)?
        .total_supply;
    let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
    Ok(Tokens {
        share,
        asset,
        total_shares,
        total_assets,
    })
}

#[derive(Debug)]
pub enum Rounding {
    Floor,
    Ceil,
}

/// Internal conversion
pub fn _convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_shares, total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Internal conversion
pub fn _convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_assets + Uint128::one(), total_shares);
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Used internally in `deposit`/`mint` functionality
pub fn _deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let this = env.contract.address.clone();
    let transfer_response = cw20_base::allowances::execute_transfer_from(
        deps.branch(),
        env.clone(),
        info.clone(),
        caller.to_string(),
        this.to_string(),
        assets,
    )?;
    let mint_response =
        cw20_base::contract::execute_mint(deps.branch(), env, info, receiver.to_string(), shares)?;
    Ok(Response::new()
        .add_submessages(transfer_response.messages)
        .add_events(transfer_response.events)
        .add_submessages(mint_response.messages)
        .add_events(mint_response.events)
        .add_attribute("action", "deposit")
        .add_attribute("depositor", caller)
        .add_attribute("receiver", receiver)
        .add_attribute("assets_transferred", assets)
        .add_attribute("shares_minted", shares))
}
