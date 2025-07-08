use cosmwasm_std::{Addr, BlockInfo, DepsMut, Env, MessageInfo, Response, Uint128};
use cw4626::{
    MaxDepositResponse, MaxMintResponse, MaxRedeemResponse, MaxWithdrawResponse,
    PreviewDepositResponse, PreviewMintResponse, PreviewRedeemResponse, PreviewWithdrawResponse,
};

use crate::{
    helpers::{_deposit, _withdraw},
    query, ContractError,
};

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    assets: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxDeposit {
            receiver: receiver.to_string(),
            assets: assets.u128(),
            max_assets: max_assets.u128(),
        });
    }
    let PreviewDepositResponse { shares } =
        query::preview_deposit(&env.contract.address, &deps.as_ref(), assets)?;
    _deposit(deps, env, info, sender, receiver, assets, shares)
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    shares: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let deps_ref = deps.as_ref();
    let MaxMintResponse { max_shares } = query::max_mint(&deps_ref, receiver.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxMint {
            receiver: receiver.to_string(),
            shares: shares.u128(),
            max_shares: max_shares.u128(),
        });
    }
    let PreviewMintResponse { assets } =
        query::preview_mint(&env.contract.address, &deps_ref, shares)?;
    _deposit(deps, env, info, sender, receiver, assets, shares)
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let this = env.contract.address.clone();
    let MaxWithdrawResponse { max_assets } =
        query::max_withdraw(&this, &deps.as_ref(), owner.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxWithdraw {
            owner: owner.to_string(),
            assets: assets.u128(),
            max_assets: max_assets.u128(),
        });
    }
    let PreviewWithdrawResponse { shares } =
        query::preview_withdraw(&this, &deps.as_ref(), assets)?;
    _withdraw(deps, env, info.sender, receiver, owner, assets, shares)
}

pub fn redeem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shares: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let MaxRedeemResponse { max_shares } = query::max_redeem(&deps.as_ref(), owner.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxRedeem {
            owner: owner.to_string(),
            shares: shares.u128(),
            max_shares: max_shares.u128(),
        });
    }
    let PreviewRedeemResponse { assets } =
        query::preview_redeem(&env.contract.address, &deps.as_ref(), shares)?;
    _withdraw(deps, env, info.sender, receiver, owner, assets, shares)
}

pub fn update_ownership(
    deps: DepsMut,
    block: BlockInfo,
    new_owner: Addr,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    cw_ownable::update_ownership(deps, &block, &new_owner, action)?;
    Ok(Response::new())
}
