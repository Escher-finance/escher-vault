#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::helpers::validate_cw20;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

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
    let deps_ref = deps.as_ref();
    let asset_info = validate_cw20(&deps_ref, &msg.underlying_token_address)?;
    let share_info = validate_cw20(&deps_ref, &msg.share_token_address)?;
    let config = Config {
        asset_address: msg.underlying_token_address,
        share_address: msg.share_token_address,
        asset_decimals: asset_info.decimals,
        share_decimals: share_info.decimals,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
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
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
    }
}

pub mod execute {
    use cosmwasm_std::{Addr, Uint128};

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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Share {} => to_json_binary(&query::share()?),
        QueryMsg::Asset {} => to_json_binary(&query::asset()?),
        QueryMsg::TotalShares {} => to_json_binary(&query::total_shares()?),
        QueryMsg::TotalAssets {} => to_json_binary(&query::total_assets()?),
        QueryMsg::ConvertToShares { assets } => to_json_binary(&query::convert_to_shares(assets)?),
        QueryMsg::ConvertToAssets { shares } => to_json_binary(&query::convert_to_assets(shares)?),
        QueryMsg::MaxDeposit { receiver } => to_json_binary(&query::max_deposit(receiver)?),
        QueryMsg::PreviewDeposit { assets } => to_json_binary(&query::preview_deposit(assets)?),
        QueryMsg::MaxMint { receiver } => to_json_binary(&query::max_mint(receiver)?),
        QueryMsg::PreviewMint { shares } => to_json_binary(&query::preview_mint(shares)?),
        QueryMsg::MaxWithdraw { owner } => to_json_binary(&query::max_withdraw(owner)?),
        QueryMsg::PreviewWithdraw { assets } => to_json_binary(&query::preview_withdraw(assets)?),
        QueryMsg::MaxRedeem { owner } => to_json_binary(&query::max_redeem(owner)?),
        QueryMsg::PreviewRedeem { shares } => to_json_binary(&query::preview_redeem(shares)?),
        QueryMsg::Ownership {} => to_json_binary(&query::ownership()?),
    }
}

pub mod query {
    use super::*;
    use cosmwasm_std::{Addr, Uint128};
    use cw4626::*;

    pub fn share() -> StdResult<ShareResponse> {
        todo!();
    }

    pub fn asset() -> StdResult<AssetResponse> {
        todo!();
    }

    pub fn total_shares() -> StdResult<TotalSharesResponse> {
        todo!();
    }

    pub fn total_assets() -> StdResult<TotalAssetsResponse> {
        todo!();
    }

    pub fn convert_to_shares(_assets: Uint128) -> StdResult<ConvertToSharesResponse> {
        todo!();
    }

    pub fn convert_to_assets(_shares: Uint128) -> StdResult<ConvertToAssetsResponse> {
        todo!();
    }

    pub fn max_deposit(_receiver: Addr) -> StdResult<MaxDepositResponse> {
        todo!();
    }

    pub fn preview_deposit(_assets: Uint128) -> StdResult<PreviewDepositResponse> {
        todo!();
    }

    pub fn max_mint(_receiver: Addr) -> StdResult<MaxMintResponse> {
        todo!();
    }

    pub fn preview_mint(_shares: Uint128) -> StdResult<PreviewMintResponse> {
        todo!();
    }

    pub fn max_withdraw(_owner: Addr) -> StdResult<MaxWithdrawResponse> {
        todo!();
    }

    pub fn preview_withdraw(_assets: Uint128) -> StdResult<PreviewWithdrawResponse> {
        todo!();
    }

    pub fn max_redeem(_owner: Addr) -> StdResult<MaxRedeemResponse> {
        todo!();
    }

    pub fn preview_redeem(_shares: Uint128) -> StdResult<PreviewRedeemResponse> {
        todo!();
    }

    pub fn ownership() -> StdResult<PreviewRedeemResponse> {
        todo!();
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
