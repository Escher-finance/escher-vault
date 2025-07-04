#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::error::ContractError;
use crate::helpers::{validate_cw20, validate_share_connected};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ASSET, SHARE};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        msg.owner.as_ref().map(|o| o.as_str()),
    )?;
    validate_cw20(&deps.querier, &msg.underlying_token_address)?;
    ASSET.save(deps.storage, &msg.underlying_token_address)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    if !matches!(
        msg,
        ExecuteMsg::ConnectShareToken { .. } | ExecuteMsg::UpdateOwnership(_)
    ) {
        validate_share_connected(deps.storage)?;
    }
    let this = env.contract.address;
    match msg {
        ExecuteMsg::Deposit { assets, receiver } => execute::deposit(assets, receiver),
        ExecuteMsg::Mint { shares, receiver } => execute::mint(shares, receiver),
        ExecuteMsg::Withdraw {
            assets,
            receiver,
            owner,
        } => execute::withdraw(assets, receiver, owner),
        ExecuteMsg::Redeem {
            shares,
            receiver,
            owner,
        } => execute::redeem(shares, receiver, owner),
        ExecuteMsg::ConnectShareToken {
            share_token_address,
        } => execute::connect_share_token(deps, info, this, share_token_address),
        ExecuteMsg::UpdateOwnership(action) => {
            execute::update_ownership(deps, &env.block, info.sender, action)
        }
    }
}

pub mod execute {
    use cosmwasm_std::{Addr, BlockInfo, Uint128};

    use super::*;

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

    pub fn redeem(
        _shares: Uint128,
        _receiver: Addr,
        _owner: Addr,
    ) -> Result<Response, ContractError> {
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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    if !matches!(msg, QueryMsg::Asset {} | QueryMsg::Ownership {}) {
        validate_share_connected(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
    }
    let this = env.contract.address;
    match msg {
        QueryMsg::Share {} => to_json_binary(&query::share(deps.storage)?),
        QueryMsg::Asset {} => to_json_binary(&query::asset(deps.storage)?),
        QueryMsg::TotalShares {} => to_json_binary(&query::total_shares(&this, &deps)?),
        QueryMsg::TotalAssets {} => to_json_binary(&query::total_assets(&this, &deps)?),
        QueryMsg::ConvertToShares { assets } => {
            to_json_binary(&query::convert_to_shares(&this, &deps, assets)?)
        }
        QueryMsg::ConvertToAssets { shares } => {
            to_json_binary(&query::convert_to_assets(&this, &deps, shares)?)
        }
        QueryMsg::MaxDeposit { receiver } => to_json_binary(&query::max_deposit(receiver)?),
        QueryMsg::PreviewDeposit { assets } => {
            to_json_binary(&query::preview_deposit(&this, &deps, assets)?)
        }
        QueryMsg::MaxMint { receiver } => to_json_binary(&query::max_mint(&deps, receiver)?),
        QueryMsg::PreviewMint { shares } => {
            to_json_binary(&query::preview_mint(&this, &deps, shares)?)
        }
        QueryMsg::MaxWithdraw { owner } => {
            to_json_binary(&query::max_withdraw(&this, &deps, owner)?)
        }
        QueryMsg::PreviewWithdraw { assets } => {
            to_json_binary(&query::preview_withdraw(&this, &deps, assets)?)
        }
        QueryMsg::MaxRedeem { owner } => to_json_binary(&query::max_redeem(&deps, owner)?),
        QueryMsg::PreviewRedeem { shares } => {
            to_json_binary(&query::preview_redeem(&this, &deps, shares)?)
        }
        QueryMsg::Ownership {} => to_json_binary(&query::ownership(deps.storage)?),
    }
}

pub mod query {
    use crate::helpers::{_convert_to_shares, query_cw20_balance, Rounding, _convert_to_assets};

    use super::*;
    use cosmwasm_std::{Addr, Storage, Uint128};
    use cw4626::*;

    pub fn share(storage: &dyn Storage) -> StdResult<ShareResponse> {
        let share = SHARE.load(storage)?;
        Ok(ShareResponse {
            share_token_address: share,
        })
    }

    pub fn asset(storage: &dyn Storage) -> StdResult<AssetResponse> {
        let asset = ASSET.load(storage)?;
        Ok(AssetResponse {
            asset_token_address: asset,
        })
    }

    pub fn total_shares(this: &Addr, deps: &Deps) -> StdResult<TotalSharesResponse> {
        let share = SHARE.load(deps.storage)?;
        let balance = query_cw20_balance(&deps.querier, &share, this)?;
        Ok(TotalSharesResponse {
            total_managed_shares: balance,
        })
    }

    pub fn total_assets(this: &Addr, deps: &Deps) -> StdResult<TotalAssetsResponse> {
        let asset = ASSET.load(deps.storage)?;
        let balance = query_cw20_balance(&deps.querier, &asset, this)?;
        Ok(TotalAssetsResponse {
            total_managed_assets: balance,
        })
    }

    pub fn convert_to_shares(
        this: &Addr,
        deps: &Deps,
        assets: Uint128,
    ) -> StdResult<ConvertToSharesResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
        Ok(ConvertToSharesResponse { shares })
    }

    pub fn convert_to_assets(
        this: &Addr,
        deps: &Deps,
        shares: Uint128,
    ) -> StdResult<ConvertToAssetsResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let assets = _convert_to_shares(total_shares, total_assets, shares, Rounding::Floor)?;
        Ok(ConvertToAssetsResponse { assets })
    }

    pub fn max_deposit(_receiver: Addr) -> StdResult<MaxDepositResponse> {
        Ok(MaxDepositResponse {
            max_assets: Uint128::MAX,
        })
    }

    pub fn preview_deposit(
        this: &Addr,
        deps: &Deps,
        assets: Uint128,
    ) -> StdResult<PreviewDepositResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
        Ok(PreviewDepositResponse { shares })
    }

    pub fn max_mint(deps: &Deps, _receiver: Addr) -> StdResult<MaxMintResponse> {
        let share = SHARE.load(deps.storage)?;
        let cw20::MinterResponse { minter: _, cap } = deps
            .querier
            .query_wasm_smart(&share, &cw20::Cw20QueryMsg::Minter {})?;
        Ok(MaxMintResponse {
            max_shares: cap.unwrap_or(Uint128::MAX),
        })
    }

    pub fn preview_mint(
        this: &Addr,
        deps: &Deps,
        shares: Uint128,
    ) -> StdResult<PreviewMintResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Ceil)?;
        Ok(PreviewMintResponse { assets })
    }

    pub fn max_withdraw(this: &Addr, deps: &Deps, owner: Addr) -> StdResult<MaxWithdrawResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let owner_shares_balance = query_cw20_balance(&deps.querier, &share, &owner)?;
        let assets = _convert_to_assets(
            total_shares,
            total_assets,
            owner_shares_balance,
            Rounding::Floor,
        )?;
        Ok(MaxWithdrawResponse { max_assets: assets })
    }

    pub fn preview_withdraw(
        this: &Addr,
        deps: &Deps,
        assets: Uint128,
    ) -> StdResult<PreviewWithdrawResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Ceil)?;
        Ok(PreviewWithdrawResponse { shares })
    }

    pub fn max_redeem(deps: &Deps, owner: Addr) -> StdResult<MaxRedeemResponse> {
        let share = SHARE.load(deps.storage)?;
        let owner_balance = query_cw20_balance(&deps.querier, &share, &owner)?;
        Ok(MaxRedeemResponse {
            max_shares: owner_balance,
        })
    }

    pub fn preview_redeem(
        this: &Addr,
        deps: &Deps,
        shares: Uint128,
    ) -> StdResult<PreviewRedeemResponse> {
        let share = SHARE.load(deps.storage)?;
        let asset = ASSET.load(deps.storage)?;
        let total_shares = query_cw20_balance(&deps.querier, &share, this)?;
        let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
        let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Floor)?;
        Ok(PreviewRedeemResponse { assets })
    }

    pub fn ownership(storage: &dyn Storage) -> StdResult<cw_ownable::Ownership<Addr>> {
        cw_ownable::get_ownership(storage)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {
        assert!(true);
    }
    // use super::*;
    // use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    // use cosmwasm_std::{coins, from_json};
    //
    // #[test]
    // fn proper_initialization() {
    //     let mut deps = mock_dependencies();
    //
    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //
    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());
    //
    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_json(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }
    //
    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies();
    //
    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //
    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //
    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_json(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }
    //
    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies();
    //
    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //
    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }
    //
    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();
    //
    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_json(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
