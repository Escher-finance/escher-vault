use crate::{
    helpers::{
        _convert_to_shares, query_cw20_balance, Rounding, Tokens, _convert_to_assets, get_tokens,
    },
    state::{ASSET, SHARE, WITHDRAWAL_SHARE_ALLOWANCES},
};

use cosmwasm_std::{Addr, BlockInfo, Deps, Order, StdResult, Storage, Uint128};
use cw20::AllowanceInfo;
use cw4626::*;
use cw_storage_plus::Bound;

const ALLOWANCE_PAGINATION_MAX_LIMIT: u32 = 50;
const ALLOWANCE_PAGINATION_DEFAULT_LIMIT: u32 = 10;

pub fn asset(storage: &dyn Storage) -> StdResult<AssetResponse> {
    let asset = ASSET.load(storage)?;
    Ok(AssetResponse {
        asset_token_address: asset,
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
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(ConvertToSharesResponse { shares })
}

pub fn convert_to_assets(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<ConvertToAssetsResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
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
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
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

pub fn preview_mint(this: &Addr, deps: &Deps, shares: Uint128) -> StdResult<PreviewMintResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Ceil)?;
    Ok(PreviewMintResponse { assets })
}

pub fn max_withdraw(this: &Addr, deps: &Deps, owner: Addr) -> StdResult<MaxWithdrawResponse> {
    let Tokens {
        total_shares,
        total_assets,
        share,
        ..
    } = get_tokens(this, deps)?;
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
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
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
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Floor)?;
    Ok(PreviewRedeemResponse { assets })
}

pub fn ownership(storage: &dyn Storage) -> StdResult<cw_ownable::Ownership<Addr>> {
    cw_ownable::get_ownership(storage)
}

pub fn withdrawal_share_allowance(
    storage: &dyn Storage,
    block: &BlockInfo,
    owner: Addr,
    spender: Addr,
) -> StdResult<WithdrawalShareAllowanceResponse> {
    let allowance = WITHDRAWAL_SHARE_ALLOWANCES
        .may_load(storage, (&owner, &spender))?
        .filter(|allow| !allow.expires.is_expired(block))
        .unwrap_or_default();
    Ok(allowance)
}

pub fn all_withdrawal_share_allowances(
    storage: &dyn Storage,
    owner: Addr,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<AllWithdrawalShareAllowancesResponse> {
    let limit = limit
        .unwrap_or(ALLOWANCE_PAGINATION_DEFAULT_LIMIT)
        .min(ALLOWANCE_PAGINATION_MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.as_bytes().to_vec()));
    let allowances = WITHDRAWAL_SHARE_ALLOWANCES
        .prefix(&owner)
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, allow)| AllowanceInfo {
                spender: addr.into(),
                allowance: allow.allowance,
                expires: allow.expires,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(AllWithdrawalShareAllowancesResponse { allowances })
}
