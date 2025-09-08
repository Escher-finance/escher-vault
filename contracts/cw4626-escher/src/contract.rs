#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw4626_base::execute as cw4626_base_executes;
use cw4626_base::query as cw4626_base_queries;

use crate::asset::query_asset_info_decimals;
use crate::error::ContractError;
use crate::helpers::validate_addrs;
use crate::helpers::PreviewDepositKind;
use crate::msg::MigrateMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::staking::EscherHubQueryMsg;
use crate::staking::EscherHubStakingLiquidity;
use crate::state::{AccessControlRole, ACCESS_CONTROL, UNDERLYING_ASSET, UNDERLYING_DECIMALS};
use crate::tower::{init_oracle_prices, update_tower_config};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let underlying_decimals =
        query_asset_info_decimals(&deps.querier, msg.underlying_token.clone())?;
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
    UNDERLYING_ASSET.save(deps.storage, &msg.underlying_token)?;
    UNDERLYING_DECIMALS.save(deps.storage, &underlying_decimals)?;

    // Save staking contract address if provided
    if let Some(staking_contract) = msg.staking_contract {
        let _ = deps
            .querier
            .query_wasm_smart::<EscherHubStakingLiquidity>(
                staking_contract.clone(),
                &EscherHubQueryMsg::StakingLiquidity {},
            )
            .map_err(|_| ContractError::InvalidStakingContract {})?;
        crate::state::STAKING_CONTRACT.save(deps.storage, &staking_contract)?;
    }

    ACCESS_CONTROL.save(
        deps.storage,
        AccessControlRole::Manager {}.key(),
        &validate_addrs(msg.managers.into_iter())?,
    )?;
    ACCESS_CONTROL.save(
        deps.storage,
        AccessControlRole::Oracle {}.key(),
        &validate_addrs(msg.oracles.into_iter())?,
    )?;
    let tower_config = update_tower_config(
        deps.branch(),
        msg.tower_incentives,
        msg.lp,
        msg.slippage_tolerance,
        msg.incentives,
        msg.underlying_token,
    )?;
    init_oracle_prices(deps, &tower_config)?;
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
    Ok(match msg {
        ExecuteMsg::AddToRole { role, address } => {
            crate::execute::add_to_role(deps, sender, role, address)?
        }
        ExecuteMsg::RemoveFromRole { role, address } => {
            crate::execute::remove_from_role(deps, sender, role, address)?
        }
        ExecuteMsg::OracleUpdatePrices { prices } => {
            crate::execute::oracle_update_prices(deps, sender, prices)?
        }
        ExecuteMsg::Bond {
            amount,
            salt,
            slippage,
        } => crate::execute::bond(deps, env, info, amount, salt, slippage)?,
        ExecuteMsg::Unbond { amount } => crate::execute::unbond(deps, env, info, amount)?,
        ExecuteMsg::AddLiquidity {
            underlying_token_amount,
        } => crate::execute::add_liquidity(deps, env, info, underlying_token_amount)?,
        ExecuteMsg::RemoveLiquidity { lp_token_amount } => {
            crate::execute::remove_liquidity(deps, env, info, lp_token_amount)?
        }
        ExecuteMsg::ClaimIncentives {} => crate::execute::claim_incentives(deps, info)?,
        ExecuteMsg::Swap { amount, asset_info } => {
            crate::execute::swap(deps, env, info, amount, asset_info)?
        }
        //
        // CW4626
        //
        ExecuteMsg::Deposit { assets, receiver } => {
            crate::execute::deposit(deps, env, info, assets, receiver)?
        }
        ExecuteMsg::Mint { shares, receiver } => {
            crate::execute::mint(deps, env, info, shares, receiver)?
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
        ExecuteMsg::RequestRedeem {
            shares,
            receiver,
            owner,
        } => crate::execute::request_redeem(deps, env, info, shares, receiver, owner)?,
        ExecuteMsg::CompleteRedemptionWithDistribution {
            redemption_id,
            tx_hash,
        } => crate::execute::complete_redemption_with_distribution(
            deps,
            env,
            info,
            redemption_id,
            tx_hash,
        )?,
        ExecuteMsg::Receive(cw20_receive_msg) => {
            crate::execute::receive(deps, env, info, sender, cw20_receive_msg)?
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
    let this = env.contract.address.clone();
    match msg {
        QueryMsg::GitInfo {} => to_json_binary(&crate::query::git_info()?),
        QueryMsg::Config {} => to_json_binary(&crate::query::config(&deps)?),
        QueryMsg::Role { kind } => to_json_binary(&crate::query::role(&deps, kind)?),
        QueryMsg::OracleTokensList {} => to_json_binary(&crate::query::oracle_tokens_list(&deps)?),
        QueryMsg::OraclePrices {} => to_json_binary(&crate::query::oracle_prices(&deps)?),
        QueryMsg::ExchangeRate {} => to_json_binary(&crate::query::exchange_rate(&this, &deps)?),
        QueryMsg::LpPosition {} => to_json_binary(&crate::query::lp_position(&this, &deps)?),
        QueryMsg::AllPendingIncentives {} => {
            to_json_binary(&crate::query::all_pending_incentives(&this, &deps)?)
        }
        //
        // Redemption System
        //
        QueryMsg::RedemptionRequest { id } => {
            to_json_binary(&crate::query::redemption_request(&deps, id)?)
        }
        QueryMsg::UserRedemptionRequests { user } => {
            to_json_binary(&crate::query::user_redemption_requests(&deps, user)?)
        }
        QueryMsg::PreviewRedeemMultiAsset { shares } => to_json_binary(
            &crate::query::preview_redeem_multi_asset(deps, shares, env.contract.address.clone())?,
        ),
        QueryMsg::RedemptionStats => to_json_binary(&crate::query::redemption_stats(deps)?),
        //
        // CW4626
        //
        QueryMsg::Asset {} => to_json_binary(&crate::query::asset(&deps)?),
        QueryMsg::TotalAssets {} => to_json_binary(&crate::query::total_assets(&deps, this)?),
        QueryMsg::ConvertToShares { assets } => {
            to_json_binary(&crate::query::convert_to_shares(&this, &deps, assets)?)
        }
        QueryMsg::ConvertToAssets { shares } => {
            to_json_binary(&crate::query::convert_to_assets(&this, &deps, shares)?)
        }
        QueryMsg::MaxDeposit { receiver } => to_json_binary(&crate::query::max_deposit(receiver)?),
        QueryMsg::PreviewDeposit { assets } => to_json_binary(&crate::query::preview_deposit(
            &this,
            &deps,
            assets,
            PreviewDepositKind::OnlyQuery {},
        )?),
        QueryMsg::MaxMint { receiver } => to_json_binary(&crate::query::max_mint(receiver)?),
        QueryMsg::PreviewMint { shares } => {
            to_json_binary(&crate::query::preview_mint(&this, &deps, shares)?)
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
