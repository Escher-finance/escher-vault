use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use cw4626_base::helpers::generate_deposit_response;

use crate::{
    asset_info::assert_send_asset_to_contract, state::UNDERLYING_ASSET,
    tower::calculate_total_assets, ContractError,
};

#[derive(Debug)]
pub struct Tokens {
    pub share: AssetInfo,
    pub asset: AssetInfo,
    pub total_shares: Uint128,
    pub total_assets: Uint128,
}

pub fn get_tokens(this: &Addr, deps: &Deps) -> StdResult<Tokens> {
    let share = AssetInfo::Token {
        contract_addr: this.clone(),
    };
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let total_shares = cw20_base::state::TOKEN_INFO
        .load(deps.storage)?
        .total_supply;
    let total_assets = calculate_total_assets(&deps.querier, deps.storage, this.clone())
        .map_err(|err| StdError::generic_err(format!("Failed to calculate total assets: {}", err)))?;
    Ok(Tokens {
        share,
        asset,
        total_shares,
        total_assets,
    })
}

#[derive(Debug)]
pub enum Rounding {
    Floor,
    Ceil,
}

/// Internal conversion with input validation
pub fn _convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    // CRITICAL: Validate inputs to prevent edge cases
    if assets.is_zero() {
        return Ok(Uint128::zero());
    }
    
    let frac = (total_shares + Uint128::one(), total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(format!("Share conversion failed: {}", e)))
}

/// Internal conversion with input validation
pub fn _convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    // CRITICAL: Validate inputs to prevent edge cases
    if shares.is_zero() {
        return Ok(Uint128::zero());
    }
    
    let frac = (total_assets + Uint128::one(), total_shares + Uint128::one());
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(format!("Asset conversion failed: {}", e)))
}

/// Pass `true` in `via_receive` in order to fix calculation when using ReceiveMsg
pub fn _preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
    via_receive: bool,
) -> StdResult<cw4626::PreviewDepositResponse> {
    // CRITICAL: Validate input amount
    if assets.is_zero() {
        return Ok(cw4626::PreviewDepositResponse { shares: Uint128::zero() });
    }
    
    let Tokens {
        total_shares,
        mut total_assets,
        ..
    } = get_tokens(this, deps)?;
    
    if via_receive {
        // CRITICAL: Prevent underflow by checking if assets > total_assets
        if assets > total_assets {
            return Err(StdError::generic_err(
                "Assets amount exceeds total assets - potential underflow"
            ));
        }
        total_assets -= assets;
    }
    
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(cw4626::PreviewDepositResponse { shares })
}

// Internal unchecked `mint` with input validation
pub fn _mint(deps: DepsMut, recipient: String, amount: Uint128) -> Result<(), ContractError> {
    // CRITICAL: Validate inputs
    if amount.is_zero() {
        return Ok(()); // No-op for zero amount
    }
    
    if recipient.is_empty() {
        return Err(ContractError::empty_input("recipient"));
    }
    
    let mut config = cw20_base::state::TOKEN_INFO.load(deps.storage)?;

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::ShareCw20Error(
                cw20_base::ContractError::CannotExceedCap {},
            ));
        }
    }
    cw20_base::state::TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    cw20_base::state::BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    Ok(())
}

/// Used internally in `deposit`/`mint` functionality with input validation
pub fn _deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    // CRITICAL: Validate inputs
    if assets.is_zero() {
        return Err(ContractError::invalid_amount(assets, "assets cannot be zero"));
    }
    
    if shares.is_zero() {
        return Err(ContractError::invalid_amount(shares, "shares cannot be zero"));
    }
    
    if receiver.as_str().is_empty() {
        return Err(ContractError::empty_input("receiver"));
    }
    
    let caller = info.sender.clone();
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset = Asset {
        amount: assets,
        info: asset_info,
    };
    let transfer_msg = assert_send_asset_to_contract(info, env, asset.clone(), &deps.querier)?;
    let mut res = generate_deposit_response(&caller, &receiver, assets, shares);
    if let Some(msg) = transfer_msg {
        res = res.add_message(msg);
    }
    _mint(deps.branch(), receiver.to_string(), shares)?;
    Ok(res)
}
