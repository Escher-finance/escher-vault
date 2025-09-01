use cosmwasm_std::{
    to_json_binary, Addr, BlockInfo, Deps, DepsMut, Env, QuerierWrapper, Response, StdError,
    StdResult, Storage, Uint128, WasmMsg, Coin, BankMsg,
};
use cw4626::{cw20, PreviewDepositResponse};

use crate::{state::{UNDERLYING_ASSET, TOKEN_TYPE, TokenType}, ContractError};

pub fn validate_cw20(
    querier: &QuerierWrapper,
    token_address: &Addr,
) -> Result<cw20::TokenInfoResponse, ContractError> {
    querier
        .query_wasm_smart::<cw20::TokenInfoResponse>(
            token_address,
            &cw20::Cw20QueryMsg::TokenInfo {},
        )
        .map_err(|_| ContractError::InvalidCw20 {
            addr: token_address.clone(),
        })
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    token: &Addr,
    user: &Addr,
) -> Result<Uint128, StdError> {
    let cw20::BalanceResponse { balance } = querier.query_wasm_smart(
        token,
        &cw20::Cw20QueryMsg::Balance {
            address: user.to_string(),
        },
    )?;
    Ok(balance)
}

#[derive(Debug)]
pub struct Tokens {
    pub share: Addr,
    pub asset: Addr,
    pub total_shares: Uint128,
    pub total_assets: Uint128,
}

pub fn get_tokens(this: &Addr, deps: &Deps) -> StdResult<Tokens> {
    let share = this.clone();
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let total_shares = cw20_base::state::TOKEN_INFO
        .load(deps.storage)?
        .total_supply;
    let total_assets = query_cw20_balance(&deps.querier, &asset, this)?;
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

/// Internal conversion
pub fn _convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_shares + Uint128::one(), total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Internal conversion
pub fn _convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_assets + Uint128::one(), total_shares + Uint128::one());
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

pub fn generate_deposit_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("depositor", caller)
        .add_attribute("receiver", receiver)
        .add_attribute("assets_transferred", assets)
        .add_attribute("shares_minted", shares)
}

pub fn generate_withdraw_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("withdrawer", caller)
        .add_attribute("receiver", receiver)
        .add_attribute("assets_received", assets)
        .add_attribute("shares_burned", shares)
}

/// Pass `true` in `via_receive` in order to fix calculation when using ReceiveMsg
pub fn _preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
    via_receive: bool,
) -> StdResult<PreviewDepositResponse> {
    let Tokens {
        total_shares,
        mut total_assets,
        ..
    } = get_tokens(this, deps)?;
    if via_receive {
        total_assets -= assets;
    }
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(PreviewDepositResponse { shares })
}

/// Used internally in `deposit`/`mint` functionality
pub fn _deposit(
    mut deps: DepsMut,
    env: Env,
    caller: Addr,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let this = env.contract.address.clone();
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let transfer_from_msg = WasmMsg::Execute {
        contract_addr: asset.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
            owner: caller.to_string(),
            recipient: this.to_string(),
            amount: assets,
        })?,
        funds: vec![],
    };
    _mint(deps.branch(), receiver.to_string(), shares)?;
    Ok(
        generate_deposit_response(&caller, &receiver, assets, shares)
            .add_message(transfer_from_msg),
    )
}

/// Used internally in `deposit_native`/`mint_native` functionality
pub fn _deposit_native(
    mut deps: DepsMut,
    env: Env,
    caller: Addr,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
    funds: Vec<Coin>,
) -> Result<Response, ContractError> {
    let token_type = TOKEN_TYPE.load(deps.storage)?;
    match token_type {
        TokenType::Native { denom } => {
            // Find the coin with the matching denom
            let coin = funds.iter()
                .find(|c| c.denom == denom)
                .ok_or_else(|| ContractError::InsufficientFunds {})?;
            
            if coin.amount < assets {
                return Err(ContractError::InsufficientFunds {});
            }
            
            // Mint shares to receiver
            _mint(deps.branch(), receiver.to_string(), shares)?;
            
            Ok(generate_deposit_response(&caller, &receiver, assets, shares))
        },
        TokenType::Cw20 { .. } => {
            Err(ContractError::InvalidTokenType {})
        }
    }
}

/// Used internally in `withdraw`/`redeem` functionality
pub fn _withdraw(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    receiver: Addr,
    owner: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    if caller != owner {
        _deduct_allowance(deps.storage, &env.block, &owner, &caller, shares)?;
    }
    _burn(deps, owner, shares)?;
    let transfer_msg = WasmMsg::Execute {
        contract_addr: asset.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: receiver.to_string(),
            amount: assets,
        })?,
        funds: vec![],
    };
    Ok(generate_withdraw_response(&caller, &receiver, assets, shares).add_message(transfer_msg))
}

/// Used internally in `withdraw_native`/`redeem_native` functionality
pub fn _withdraw_native(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    receiver: Addr,
    owner: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let token_type = TOKEN_TYPE.load(deps.storage)?;
    match token_type {
        TokenType::Native { denom } => {
            if caller != owner {
                _deduct_allowance(deps.storage, &env.block, &owner, &caller, shares)?;
            }
            _burn(deps, owner, shares)?;
            
            let transfer_msg = BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin {
                    denom,
                    amount: assets,
                }],
            };
            
            Ok(generate_withdraw_response(&caller, &receiver, assets, shares)
                .add_message(transfer_msg))
        },
        TokenType::Cw20 { .. } => {
            Err(ContractError::InvalidTokenType {})
        }
    }
}

// Internal unchecked `deduct_allowance`
pub fn _deduct_allowance(
    storage: &mut dyn Storage,
    block: &BlockInfo,
    owner: &Addr,
    spender: &Addr,
    amount: Uint128,
) -> Result<cw20::AllowanceResponse, ContractError> {
    if spender == owner {
        return Err(ContractError::ShareCw20Error(
            cw20_base::ContractError::CannotSetOwnAccount {},
        ));
    }
    Ok(cw20_base::allowances::deduct_allowance(
        storage, owner, spender, block, amount,
    )?)
}

// Internal unchecked `mint`
pub fn _mint(deps: DepsMut, recipient: String, amount: Uint128) -> Result<(), ContractError> {
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

// Internal unchecked `burn`
pub fn _burn(deps: DepsMut, user: Addr, amount: Uint128) -> Result<(), ContractError> {
    // lower balance
    cw20_base::state::BALANCES.update(
        deps.storage,
        &user,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    cw20_base::state::TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, MockStorage},
        Addr, Uint128,
    };

    use super::*;

    #[test]
    fn internal_deduct_allowance_must_not_work_on_self() {
        let mut storage = MockStorage::new();
        let env = mock_env();
        let user = Addr::unchecked("user");
        let amount = Uint128::one();
        assert_eq!(
            _deduct_allowance(&mut storage, &env.block, &user, &user, amount).unwrap_err(),
            ContractError::ShareCw20Error(cw20_base::ContractError::CannotSetOwnAccount {}),
        );
    }

    #[test]
    fn internal_mint_must_enforce_cap() {
        let mut deps = mock_dependencies();
        let deps_mut = deps.as_mut();
        let cap = Uint128::new(10);
        cw20_base::state::TOKEN_INFO
            .save(
                deps_mut.storage,
                &cw20_base::state::TokenInfo {
                    name: String::new(),
                    symbol: String::new(),
                    decimals: 6,
                    total_supply: Uint128::zero(),
                    mint: Some(cw20_base::state::MinterData {
                        minter: Addr::unchecked("minter"),
                        cap: Some(cap),
                    }),
                },
            )
            .unwrap();
        assert_eq!(
            _mint(deps_mut, "user".to_string(), cap + Uint128::one()).unwrap_err(),
            ContractError::ShareCw20Error(cw20_base::ContractError::CannotExceedCap {}),
        );
    }

    #[test]
    fn convert_to_assets_must_error_if_overflow_occurs() {
        assert!(_convert_to_assets(
            Uint128::one(),
            Uint128::MAX - Uint128::one(),
            Uint128::MAX,
            Rounding::Floor
        )
        .is_err());
    }

    #[test]
    fn convert_to_shares_must_error_if_overflow_occurs() {
        assert!(_convert_to_shares(
            Uint128::MAX - Uint128::one(),
            Uint128::one(),
            Uint128::MAX,
            Rounding::Floor
        )
        .is_err());
    }
}
