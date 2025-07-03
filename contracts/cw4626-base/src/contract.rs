#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    todo!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { assets, receiver } => execute::deposit(),
        ExecuteMsg::Mint { shares, receiver } => execute::mint(),
        ExecuteMsg::Withdraw {
            assets,
            receiver,
            owner,
        } => execute::withdraw(),
        ExecuteMsg::Redeem {
            shares,
            receiver,
            owner,
        } => execute::redeem(),
        ExecuteMsg::TransferOwnership { new_owner } => execute::transfer_ownership(),
    }
}

pub mod execute {
    use super::*;

    pub fn deposit() -> Result<Response, ContractError> {
        todo!()
    }

    pub fn mint() -> Result<Response, ContractError> {
        todo!()
    }

    pub fn withdraw() -> Result<Response, ContractError> {
        todo!()
    }

    pub fn redeem() -> Result<Response, ContractError> {
        todo!()
    }

    pub fn transfer_ownership() -> Result<Response, ContractError> {
        todo!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Share {} => to_json_binary(&query::share()?),
        QueryMsg::Asset {} => to_json_binary(&query::asset()?),
        QueryMsg::TotalShares {} => to_json_binary(&query::total_shares()?),
        QueryMsg::TotalAssets {} => to_json_binary(&query::total_assets()?),
        QueryMsg::ConvertToShares { assets } => to_json_binary(&query::convert_to_shares()?),
        QueryMsg::ConvertToAssets { shares } => to_json_binary(&query::convert_to_assets()?),
        QueryMsg::MaxDeposit { receiver } => to_json_binary(&query::max_deposit()?),
        QueryMsg::PreviewDeposit { assets } => to_json_binary(&query::preview_deposit()?),
        QueryMsg::MaxMint { receiver } => to_json_binary(&query::max_mint()?),
        QueryMsg::PreviewMint { shares } => to_json_binary(&query::preview_mint()?),
        QueryMsg::MaxWithdraw { owner } => to_json_binary(&query::max_withdraw()?),
        QueryMsg::PreviewWithdraw { assets } => to_json_binary(&query::preview_withdraw()?),
        QueryMsg::MaxRedeem { owner } => to_json_binary(&query::max_redeem()?),
        QueryMsg::PreviewRedeem { shares } => to_json_binary(&query::preview_redeem()?),
        QueryMsg::Ownership {} => to_json_binary(&query::ownership()?),
    }
}

pub mod query {
    use super::*;
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

    pub fn convert_to_shares() -> StdResult<ConvertToSharesResponse> {
        todo!();
    }

    pub fn convert_to_assets() -> StdResult<ConvertToAssetsResponse> {
        todo!();
    }

    pub fn max_deposit() -> StdResult<MaxDepositResponse> {
        todo!();
    }

    pub fn preview_deposit() -> StdResult<PreviewDepositResponse> {
        todo!();
    }

    pub fn max_mint() -> StdResult<MaxMintResponse> {
        todo!();
    }

    pub fn preview_mint() -> StdResult<PreviewMintResponse> {
        todo!();
    }

    pub fn max_withdraw() -> StdResult<MaxWithdrawResponse> {
        todo!();
    }

    pub fn preview_withdraw() -> StdResult<PreviewWithdrawResponse> {
        todo!();
    }

    pub fn max_redeem() -> StdResult<MaxRedeemResponse> {
        todo!();
    }

    pub fn preview_redeem() -> StdResult<PreviewRedeemResponse> {
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
