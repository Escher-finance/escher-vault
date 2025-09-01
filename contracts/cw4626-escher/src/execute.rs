use std::collections::HashMap;

use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Response, Uint128};
use cw4626_base::query;

use crate::{
    access_control::only_role,
    staking::EscherHubExecuteMsg,
    state::{AccessControlRole, ACCESS_CONTROL, STAKING_CONTRACT},
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
    let total_assets_response = query::total_assets(&env.contract.address, &deps.as_ref())?;
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

    Ok(Response::new()
        .add_message(bond_msg)
        .add_attribute("action", "bond")
        .add_attribute("sender", sender.to_string())
        .add_attribute("expected", expected.to_string())
        .add_attribute("staking_contract", staking_contract.to_string()))
}
