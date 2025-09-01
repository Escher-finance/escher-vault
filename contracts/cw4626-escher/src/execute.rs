use cosmwasm_std::{Addr, Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    access_control::only_role,
    helpers::_deposit,
    query,
    responses::generate_bond_response,
    staking::EscherHubExecuteMsg,
    state::{AccessControlRole, PricesMap, ACCESS_CONTROL, STAKING_CONTRACT},
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
    prices: PricesMap,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Oracle {})?;
    update_and_validate_prices(deps, prices)?;
    Ok(Response::new())
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
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;

    // Get the current total assets in the vault
    let total_assets_response = query::total_assets(&deps.as_ref(), env.contract.address)?;
    let total_assets = total_assets_response.total_managed_assets;

    // Validate that we have enough assets to bond
    if total_assets < expected {
        return Err(ContractError::InsufficientFunds {});
    }

    // Create the bond message for the staking contract
    let bond_msg = cosmwasm_std::WasmMsg::Execute {
        contract_addr: staking_contract.to_string(),
        msg: cosmwasm_std::to_json_binary(&EscherHubExecuteMsg::Bond {
            slippage,
            expected,
            recipient,
            recipient_channel_id,
            salt,
        })?,
        funds: vec![],
    };

    Ok(generate_bond_response(&sender, expected, &staking_contract).add_message(bond_msg))
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let cw4626::MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
    if assets > max_assets {
        return Err(cw4626_base::ContractError::ExceededMaxDeposit {
            receiver: receiver.clone(),
            assets,
            max_assets,
        }
        .into());
    }
    let cw4626::PreviewDepositResponse { shares } =
        query::preview_deposit(&env.contract.address, &deps.as_ref(), assets)?;
    _deposit(deps, env, info, receiver, assets, shares)
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shares: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let deps_ref = deps.as_ref();
    let cw4626::MaxMintResponse { max_shares } = query::max_mint(receiver.clone())?;
    if shares > max_shares {
        return Err(cw4626_base::ContractError::ExceededMaxMint {
            receiver: receiver.clone(),
            shares,
            max_shares,
        }
        .into());
    }
    let cw4626::PreviewMintResponse { assets } =
        query::preview_mint(&env.contract.address, &deps_ref, shares)?;
    _deposit(deps, env, info, receiver, assets, shares)
}
