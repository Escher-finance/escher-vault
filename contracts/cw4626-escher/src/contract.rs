#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::asset::query_asset_info_decimals;
use crate::error::ContractResult;
use crate::helpers::internal_update_minimum_deposit;
use crate::helpers::validate_addrs;
use crate::helpers::PreviewDepositKind;
use crate::msg::MigrateMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::staking::validate_and_store_staking_contract;
use crate::state::PausedStatus;
use crate::state::PAUSED_STATUS;
use crate::state::{
    AccessControlRole, EntryFeeConfig, ACCESS_CONTROL, ENTRY_FEE_CONFIG, UNDERLYING_ASSET,
    UNDERLYING_DECIMALS,
};
use crate::tower::{init_oracle_prices, update_tower_config};

/// # Errors
/// Will return error if migrate fails
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::new())
}

/// # Errors
/// Will return error if instantiate fails
#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::too_many_lines)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
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
        &mut deps.branch(),
        msg.tower_incentives,
        &msg.lp,
        msg.slippage_tolerance,
        msg.incentives,
        msg.underlying_token,
    )?;

    // Save staking contract address if provided
    if let Some(staking_contract) = msg.staking_contract {
        validate_and_store_staking_contract(
            &mut deps,
            &staking_contract,
            &tower_config.lp_other_asset,
        )?;
    }

    init_oracle_prices(&mut deps.branch(), &tower_config)?;

    // Initialize entry fee configuration
    ENTRY_FEE_CONFIG.save(
        deps.storage,
        &EntryFeeConfig {
            fee_rate: msg.entry_fee_rate.unwrap_or_default(),
            fee_recipient: msg.entry_fee_recipient,
        },
    )?;

    internal_update_minimum_deposit(&mut deps, msg.minimum_deposit)?;

    PAUSED_STATUS.save(deps.storage, &PausedStatus::NotPaused {})?;

    Ok(Response::new())
}

/// # Errors
/// Will return error if execute fails
#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::too_many_lines)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    let sender = info.sender.clone();
    Ok(match msg {
        ExecuteMsg::AddToRole { role, address } => {
            crate::execute::add_to_role(&mut deps, &sender, role, &address)?
        }
        ExecuteMsg::RemoveFromRole { role, address } => {
            crate::execute::remove_from_role(&mut deps, &sender, role, &address)?
        }
        ExecuteMsg::OracleUpdatePrices { prices } => {
            crate::execute::oracle_update_prices(&mut deps, &sender, &prices)?
        }
        ExecuteMsg::UpdateStakingContract { address } => {
            crate::execute::update_staking_contract(&mut deps, &info, &address)?
        }
        ExecuteMsg::UpdateMinimumDeposit { amount } => {
            crate::execute::update_minimum_deposit(&mut deps, &info, amount)?
        }
        ExecuteMsg::UpdatePausedStatus { status } => {
            crate::execute::update_paused_status(&mut deps, &info, &status)?
        }
        ExecuteMsg::Bond {
            amount,
            salt,
            slippage,
        } => crate::execute::bond(&mut deps, &env, &info, amount, salt, slippage)?,
        ExecuteMsg::Unbond { amount } => crate::execute::unbond(&mut deps, &env, &info, amount)?,
        ExecuteMsg::AddLiquidity {
            underlying_token_amount,
        } => crate::execute::add_liquidity(&mut deps, &env, &info, underlying_token_amount)?,
        ExecuteMsg::RemoveLiquidity { lp_token_amount } => {
            crate::execute::remove_liquidity(&mut deps, &env, &info, lp_token_amount)?
        }
        ExecuteMsg::ClaimIncentives {} => crate::execute::claim_incentives(&mut deps, &info)?,
        ExecuteMsg::Swap { amount, asset_info } => {
            crate::execute::swap(&mut deps, &env, &info, amount, asset_info)?
        }
        //
        // CW4626
        //
        ExecuteMsg::Deposit { assets, receiver } => {
            crate::execute::deposit(&mut deps, &env, &info, assets, &receiver)?
        }
        ExecuteMsg::RequestRedeem {
            shares,
            receiver,
            owner,
        } => crate::execute::request_redeem(deps, &env, &info, shares, &receiver, &owner)?,
        ExecuteMsg::CompleteRedemption {
            redemption_id,
            tx_hash,
        } => crate::execute::complete_redemption_with_distribution(
            deps,
            &env,
            &info,
            redemption_id,
            &tx_hash,
        )?,
        ExecuteMsg::Receive(cw20_receive_msg) => {
            crate::execute::receive(&mut deps, &env, &info, sender, &cw20_receive_msg)?
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
        // TODO: remove this
        // DEV only
        ExecuteMsg::TokenOrderV2 {
            ucs03,
            channel_id,
            receiver,
            amount,
            denom,
            quote_token,
            salt,
        } => crate::execute::token_order_v2(
            &mut deps,
            &env,
            &ucs03,
            channel_id,
            &receiver,
            amount,
            denom,
            &quote_token,
            &salt,
        )?,
    })
}

/// # Errors
/// Will return error if query fails
#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::needless_pass_by_value)]
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
        QueryMsg::Paused {} => to_json_binary(&crate::query::paused(&deps)?),
        //
        // Redemption System
        //
        QueryMsg::RedemptionRequest { id } => {
            to_json_binary(&crate::query::redemption_request(&deps, id)?)
        }
        QueryMsg::UserRedemptionRequests { user } => {
            to_json_binary(&crate::query::user_redemption_requests(&deps, &user)?)
        }
        QueryMsg::PreviewRedeemMultiAsset { shares } => to_json_binary(
            &crate::query::preview_redeem_multi_asset(deps, shares, &this)?,
        ),
        QueryMsg::RedemptionStats => to_json_binary(&crate::query::redemption_stats(deps)?),
        QueryMsg::AllRedemptionRequests { start_after, limit } => to_json_binary(
            &crate::query::all_redemption_requests(&deps, start_after, limit)?,
        ),
        //
        // CW4626
        //
        QueryMsg::Asset {} => to_json_binary(&crate::query::asset(&deps)?),
        QueryMsg::TotalAssets {} => to_json_binary(&crate::query::total_assets(&deps, &this)?),
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
        QueryMsg::MaxRedeem { owner } => to_json_binary(&crate::query::max_redeem(&deps, &owner)?),
        QueryMsg::PreviewRedeem { shares } => {
            to_json_binary(&crate::query::preview_redeem(&this, &deps, shares)?)
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
