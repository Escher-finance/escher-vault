#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw4626::cw20;

use crate::error::ContractError;
use crate::execute;
use crate::helpers::validate_cw20;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query;
use crate::state::{UNDERLYING_ASSET, UNDERLYING_DECIMALS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let cw20::TokenInfoResponse {
        decimals: underlying_decimals,
        ..
    } = validate_cw20(&deps.querier, &msg.underlying_token_address)?;
    cw20_base::contract::instantiate(
        deps.branch(),
        env,
        info,
        cw20_base::msg::InstantiateMsg {
            name: msg.share_name,
            symbol: msg.share_symbol,
            decimals: underlying_decimals,
            initial_balances: vec![],
            mint: None,
            marketing: msg.share_marketing,
        },
    )?;
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        msg.owner.as_ref().map(|o| o.as_str()),
    )?;
    UNDERLYING_ASSET.save(deps.storage, &msg.underlying_token_address)?;
    UNDERLYING_DECIMALS.save(deps.storage, &underlying_decimals)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    match msg {
        //
        // CW4626
        //
        ExecuteMsg::Deposit { assets, receiver } => {
            execute::deposit(deps, env, sender, assets, receiver)
        }
        ExecuteMsg::Mint { shares, receiver } => execute::mint(deps, env, sender, shares, receiver),
        ExecuteMsg::Withdraw {
            assets,
            receiver,
            owner,
        } => execute::withdraw(deps, env, sender, assets, receiver, owner),
        ExecuteMsg::Redeem {
            shares,
            receiver,
            owner,
        } => execute::redeem(deps, env, sender, shares, receiver, owner),
        ExecuteMsg::UpdateOwnership(action) => execute::update_ownership(deps, env, sender, action),
        ExecuteMsg::Receive(cw20_receive_msg) => {
            execute::receive(deps, env, sender, cw20_receive_msg)
        }
        //
        // CW20
        //
        ExecuteMsg::Transfer { recipient, amount } => Ok(cw20_base::contract::execute_transfer(
            deps, env, info, recipient, amount,
        )?),
        ExecuteMsg::Burn { amount } => {
            Ok(cw20_base::contract::execute_burn(deps, env, info, amount)?)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(cw20_base::contract::execute_send(
            deps, env, info, contract, amount, msg,
        )?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(cw20_base::allowances::execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(cw20_base::allowances::execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(cw20_base::allowances::execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => Ok(cw20_base::allowances::execute_burn_from(
            deps, env, info, owner, amount,
        )?),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(cw20_base::allowances::execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => Ok(cw20_base::contract::execute_update_marketing(
            deps,
            env,
            info,
            project,
            description,
            marketing,
        )?),
        ExecuteMsg::UploadLogo(logo) => Ok(cw20_base::contract::execute_upload_logo(
            deps, env, info, logo,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let this = env.contract.address;
    match msg {
        //
        // CW4626
        //
        QueryMsg::Asset {} => to_json_binary(&query::asset(deps.storage)?),
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
        QueryMsg::MaxMint { receiver } => to_json_binary(&query::max_mint(receiver)?),
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
