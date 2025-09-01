use std::collections::HashMap;

use cosmwasm_std::{Addr, Decimal, DepsMut, Response};

use crate::{
    access_control::only_role,
    state::{AccessControlRole, ACCESS_CONTROL},
    tower::update_and_validate_prices,
    ContractError,
};

pub fn update_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.save(deps.storage, role.key(), &address)?;
    Ok(Response::new())
}

pub fn oracle_update_prices(
    deps: DepsMut,
    sender: Addr,
    prices: HashMap<String, Decimal>,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Oracle {})?;
    update_and_validate_prices(deps, prices)?;
    Ok(Response::new())
}

// Native token operations
pub fn deposit_native(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    receiver: Addr,
    funds: Vec<cosmwasm_std::Coin>,
) -> Result<Response, ContractError> {
    let token_type = crate::state::TOKEN_TYPE.load(deps.storage)?;
    match token_type {
        crate::state::TokenType::Native { denom } => {
            // Find the coin with the matching denom
            let coin = funds
                .iter()
                .find(|c| c.denom == denom)
                .ok_or_else(|| ContractError::InsufficientFunds {})?;

            let assets = coin.amount;
            let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
            if assets > max_assets {
                return Err(ContractError::ExceededMaxDeposit {
                    receiver: receiver.clone(),
                    assets,
                    max_assets,
                });
            }
            let PreviewDepositResponse { shares } =
                query::preview_deposit(&env.contract.address, &deps.as_ref(), assets)?;
            crate::helpers::_deposit_native(deps, env, sender, receiver, assets, shares, funds)
        }
        crate::state::TokenType::Cw20 { .. } => Err(ContractError::InvalidTokenType {}),
    }
}

pub fn mint_native(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    shares: Uint128,
    receiver: Addr,
    funds: Vec<cosmwasm_std::Coin>,
) -> Result<Response, ContractError> {
    let deps_ref = deps.as_ref();
    let MaxMintResponse { max_shares } = query::max_mint(receiver.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxMint {
            receiver: receiver.clone(),
            shares,
            max_shares,
        });
    }
    let PreviewMintResponse { assets } =
        query::preview_mint(&env.contract.address, &deps_ref, shares)?;
    crate::helpers::_deposit_native(deps, env, sender, receiver, assets, shares, funds)
}

pub fn withdraw_native(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    assets: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let MaxWithdrawResponse { max_assets } =
        query::max_withdraw(&env.contract.address, &deps.as_ref(), owner.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxWithdraw {
            owner: owner.clone(),
            assets,
            max_assets,
        });
    }
    let PreviewWithdrawResponse { shares } =
        query::preview_withdraw(&env.contract.address, &deps.as_ref(), assets)?;
    crate::helpers::_withdraw_native(deps, env, sender, receiver, owner, assets, shares)
}

pub fn redeem_native(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    shares: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let deps_ref = deps.as_ref();
    let MaxRedeemResponse { max_shares } = query::max_redeem(&deps.as_ref(), owner.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxRedeem {
            owner: owner.clone(),
            shares,
            max_shares,
        });
    }
    let PreviewRedeemResponse { assets } =
        query::preview_redeem(&env.contract.address, &deps_ref, shares)?;
    crate::helpers::_withdraw_native(deps, env, sender, receiver, owner, assets, shares)
}

pub fn bond(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    slippage: Option<Decimal>,
    expected: Uint128,
    recipient: Option<String>,
    recipient_channel_id: Option<u32>,
    salt: Option<String>,
) -> Result<Response, ContractError> {
    let staking_contract = crate::state::STAKING_CONTRACT.load(deps.storage)?;

    // Get the current total assets in the vault
    let total_assets_response = crate::query::total_assets(&env.contract.address, &deps.as_ref())?;
    let total_assets = total_assets_response.total_managed_assets;

    // Validate that we have enough assets to bond
    if total_assets < expected {
        return Err(ContractError::InsufficientFunds {});
    }

    // Create the bond message for the staking contract
    let bond_msg = cosmwasm_std::WasmMsg::Execute {
        contract_addr: staking_contract.to_string(),
        msg: cosmwasm_std::to_json_binary(&cw4626::Cw4626ExecuteMsg::Bond {
            slippage,
            expected,
            recipient,
            recipient_channel_id,
            salt,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(bond_msg)
        .add_attribute("action", "bond")
        .add_attribute("sender", sender.to_string())
        .add_attribute("expected", expected.to_string())
        .add_attribute("staking_contract", staking_contract.to_string()))
}
