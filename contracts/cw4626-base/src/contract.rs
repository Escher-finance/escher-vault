#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::error::ContractError;
use crate::execute;
use crate::helpers::{validate_cw20, validate_share_connected};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query;
use crate::state::ASSET;

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
    let sender = info.sender.clone();
    match msg {
        ExecuteMsg::Deposit { assets, receiver } => {
            execute::deposit(deps, this, sender, assets, receiver)
        }
        ExecuteMsg::Mint { shares, receiver } => {
            execute::mint(deps, this, sender, shares, receiver)
        }
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
            execute::update_ownership(deps, env.block, sender, action)
        }
        ExecuteMsg::IncreaseWithdrawalShareAllowance {
            spender,
            amount,
            expires,
        } => execute::increase_withdrawal_share_allowance(
            deps,
            info.sender,
            env.block,
            spender,
            amount,
            expires,
        ),
        ExecuteMsg::DecreaseWithdrawalShareAllowance {
            spender,
            amount,
            expires,
        } => execute::decrease_withdrawal_share_allowance(spender, amount, expires),
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
        QueryMsg::WithdrawalShareAllowance { owner, spender } => to_json_binary(
            &query::withdrawal_share_allowance(deps.storage, &env.block, owner, spender)?,
        ),
        QueryMsg::AllWithdrawalShareAllowances {
            owner,
            start_after,
            limit,
        } => to_json_binary(&query::all_withdrawal_share_allowances(
            deps.storage,
            owner,
            start_after,
            limit,
        )?),
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
