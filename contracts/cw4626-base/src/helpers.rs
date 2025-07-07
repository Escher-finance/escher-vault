use cosmwasm_std::{
    to_json_binary, Addr, Deps, DepsMut, QuerierWrapper, Response, StdError, StdResult, Storage,
    Uint128, WasmMsg,
};
use cw4626::cw20;

use crate::{
    state::{ASSET, SHARE},
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
    let share = SHARE.load(deps.storage)?;
    let asset = ASSET.load(deps.storage)?;
    let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
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
    deps: DepsMut,
    this: Addr,
    caller: Addr,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let share = SHARE.load(deps.storage)?;
    let asset = ASSET.load(deps.storage)?;
    let asset_transfer_msg = WasmMsg::Execute {
        contract_addr: asset.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
            owner: caller.to_string(),
            recipient: this.to_string(),
            amount: assets,
        })?,
        funds: vec![],
    };
    let share_mint_msg = WasmMsg::Execute {
        contract_addr: share.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint {
            recipient: receiver.to_string(),
            amount: shares,
        })?,
        funds: vec![],
    };
    Ok(Response::new()
        .add_message(asset_transfer_msg)
        .add_message(share_mint_msg)
        .add_attribute("action", "deposit")
        .add_attribute("share", share.to_string())
        .add_attribute("asset", asset.to_string())
        .add_attribute("depositor", caller.to_string())
        .add_attribute("receiver", receiver.to_string())
        .add_attribute("assets_transferred", assets.to_string())
        .add_attribute("shares_minted", shares.to_string()))
}

#[derive(Debug)]
pub enum AllowanceOperation {
    Increase,
    Decrease,
}
