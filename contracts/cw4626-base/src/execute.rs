use cosmwasm_std::{Addr, BlockInfo, DepsMut, Response, Uint128};
use cw4626::{MaxDepositResponse, MaxMintResponse, PreviewDepositResponse, PreviewMintResponse};

use crate::{helpers::_deposit, query, ContractError};

pub fn deposit(
    deps: DepsMut,
    this: Addr,
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
    let PreviewDepositResponse { shares } = query::preview_deposit(&this, &deps.as_ref(), assets)?;
    let response = _deposit(deps, this, sender, receiver, assets, shares)?;
    Ok(response)
}

pub fn mint(
    deps: DepsMut,
    this: Addr,
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
    let PreviewMintResponse { assets } = query::preview_mint(&this, &deps_ref, shares)?;
    let response = _deposit(deps, this, sender, receiver, assets, shares)?;
    Ok(response)
}

pub fn withdraw(
    _assets: Uint128,
    _receiver: Addr,
    _owner: Addr,
) -> Result<Response, ContractError> {
    todo!()
}

pub fn redeem(_shares: Uint128, _receiver: Addr, _owner: Addr) -> Result<Response, ContractError> {
    todo!()
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
