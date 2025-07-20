#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw4626_base::execute as cw4626_base_executes;
use cw4626_base::query as cw4626_base_queries;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(cw4626_base::contract::instantiate(deps, env, info, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    Ok(match msg {
        //
        // CW4626
        //
        ExecuteMsg::Deposit { assets, receiver } => {
            cw4626_base_executes::deposit(deps, env, sender, assets, receiver)?
        }
        ExecuteMsg::Mint { shares, receiver } => {
            cw4626_base_executes::mint(deps, env, sender, shares, receiver)?
        }
        ExecuteMsg::Withdraw {
            assets,
            receiver,
            owner,
        } => cw4626_base_executes::withdraw(deps, env, sender, assets, receiver, owner)?,
        ExecuteMsg::Redeem {
            shares,
            receiver,
            owner,
        } => cw4626_base_executes::redeem(deps, env, sender, shares, receiver, owner)?,
        ExecuteMsg::UpdateOwnership(action) => {
            cw4626_base_executes::update_ownership(deps, env, sender, action)?
        }
        ExecuteMsg::Receive(cw20_receive_msg) => {
            cw4626_base_executes::receive(deps, env, sender, cw20_receive_msg)?
        }
        //
        // CW20
        //
        ExecuteMsg::Transfer { recipient, amount } => {
            cw20_base::contract::execute_transfer(deps, env, info, recipient, amount)?
        }
        ExecuteMsg::Burn { amount } => cw20_base::contract::execute_burn(deps, env, info, amount)?,
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => cw20_base::contract::execute_send(deps, env, info, contract, amount, msg)?,
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?,
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?,
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            cw20_base::allowances::execute_transfer_from(deps, env, info, owner, recipient, amount)?
        }
        ExecuteMsg::BurnFrom { owner, amount } => {
            cw20_base::allowances::execute_burn_from(deps, env, info, owner, amount)?
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => {
            cw20_base::allowances::execute_send_from(deps, env, info, owner, contract, amount, msg)?
        }
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => cw20_base::contract::execute_update_marketing(
            deps,
            env,
            info,
            project,
            description,
            marketing,
        )?,
        ExecuteMsg::UploadLogo(logo) => {
            cw20_base::contract::execute_upload_logo(deps, env, info, logo)?
        }
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let this = env.contract.address;
    match msg {
        //
        // CW4626
        //
        QueryMsg::Asset {} => to_json_binary(&cw4626_base_queries::asset(deps.storage)?),
        QueryMsg::TotalAssets {} => {
            to_json_binary(&cw4626_base_queries::total_assets(&this, &deps)?)
        }
        QueryMsg::ConvertToShares { assets } => to_json_binary(
            &cw4626_base_queries::convert_to_shares(&this, &deps, assets)?,
        ),
        QueryMsg::ConvertToAssets { shares } => to_json_binary(
            &cw4626_base_queries::convert_to_assets(&this, &deps, shares)?,
        ),
        QueryMsg::MaxDeposit { receiver } => {
            to_json_binary(&cw4626_base_queries::max_deposit(receiver)?)
        }
        QueryMsg::PreviewDeposit { assets } => {
            to_json_binary(&cw4626_base_queries::preview_deposit(&this, &deps, assets)?)
        }
        QueryMsg::MaxMint { receiver } => to_json_binary(&cw4626_base_queries::max_mint(receiver)?),
        QueryMsg::PreviewMint { shares } => {
            to_json_binary(&cw4626_base_queries::preview_mint(&this, &deps, shares)?)
        }
        QueryMsg::MaxWithdraw { owner } => {
            to_json_binary(&cw4626_base_queries::max_withdraw(&this, &deps, owner)?)
        }
        QueryMsg::PreviewWithdraw { assets } => to_json_binary(
            &cw4626_base_queries::preview_withdraw(&this, &deps, assets)?,
        ),
        QueryMsg::MaxRedeem { owner } => {
            to_json_binary(&cw4626_base_queries::max_redeem(&deps, owner)?)
        }
        QueryMsg::PreviewRedeem { shares } => {
            to_json_binary(&cw4626_base_queries::preview_redeem(&this, &deps, shares)?)
        }
        QueryMsg::Ownership {} => to_json_binary(&cw4626_base_queries::ownership(deps.storage)?),
        //
        // CW20
        //
        QueryMsg::Balance { address } => {
            to_json_binary(&cw20_base::contract::query_balance(deps, address)?)
        }
        QueryMsg::TokenInfo {} => to_json_binary(&cw20_base::contract::query_token_info(deps)?),
        QueryMsg::Allowance { owner, spender } => to_json_binary(
            &cw20_base::allowances::query_allowance(deps, owner, spender)?,
        ),
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_json_binary(&cw20_base::enumerable::query_owner_allowances(
            deps,
            owner,
            start_after,
            limit,
        )?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_json_binary(&cw20_base::enumerable::query_spender_allowances(
            deps,
            spender,
            start_after,
            limit,
        )?),
        QueryMsg::AllAccounts { start_after, limit } => to_json_binary(
            &cw20_base::enumerable::query_all_accounts(deps, start_after, limit)?,
        ),
        QueryMsg::MarketingInfo {} => {
            to_json_binary(&cw20_base::contract::query_marketing_info(deps)?)
        }
        QueryMsg::DownloadLogo {} => {
            to_json_binary(&cw20_base::contract::query_download_logo(deps)?)
        }
    }
}
