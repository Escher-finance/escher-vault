use cosmwasm_std::{Addr, BlockInfo, DepsMut, MessageInfo, Response, Uint128};

use crate::{
    helpers::validate_cw20,
    state::{ASSET, SHARE},
    ContractError,
};

pub fn deposit(_assets: Uint128, _receiver: Addr) -> Result<Response, ContractError> {
    todo!()
}

pub fn mint(_shares: Uint128, _receiver: Addr) -> Result<Response, ContractError> {
    todo!()
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

pub fn connect_share_token(
    deps: DepsMut,
    info: MessageInfo,
    this: Addr,
    share_token_address: Addr,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let asset = ASSET.load(deps.storage)?;
    let asset_info = validate_cw20(&deps.querier, &asset)?;
    let share_info = validate_cw20(&deps.querier, &share_token_address)?;
    if asset_info.decimals != share_info.decimals {
        return Err(ContractError::DecimalsMismatch {});
    }
    let cw20::MinterResponse { minter, cap } = deps
        .querier
        .query_wasm_smart(&share_token_address, &cw20::Cw20QueryMsg::Minter {})?;
    if minter != this.to_string() {
        return Err(ContractError::InvalidSharesMinter {});
    }
    if cap == Some(Uint128::zero()) {
        return Err(ContractError::SharesMinterCapTooSmall {});
    }
    SHARE.save(deps.storage, &share_token_address)?;
    Ok(Response::new())
}

pub fn update_ownership(
    deps: DepsMut,
    block: &BlockInfo,
    new_owner: Addr,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    cw_ownable::update_ownership(deps, block, &new_owner, action)?;
    Ok(Response::new())
}
