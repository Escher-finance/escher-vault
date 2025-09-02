use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    to_json_binary, Addr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, Uint128, WasmMsg,
};
use cw4626::cw20;

use crate::{
    access_control::only_role,
    asset_info::query_asset_info_balance,
    helpers::_deposit,
    query,
    responses::generate_bond_response,
    staking::{EscherHubExecuteMsg, EscherHubQueryMsg, EscherHubStakingLiquidity},
    state::{AccessControlRole, PricesMap, ACCESS_CONTROL, STAKING_CONTRACT, UNDERLYING_ASSET},
    tower::update_and_validate_prices,
    ContractError,
};

pub fn add_to_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        let mut addrs = addrs.unwrap_or_default();
        if !addrs.contains(&address) {
            addrs.push(address);
        }
        Ok(addrs)
    })?;
    Ok(Response::new())
}

pub fn remove_from_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        Ok(addrs
            .unwrap_or_default()
            .into_iter()
            .filter(|a| a != address)
            .collect())
    })?;
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
    info: MessageInfo,
    amount: Uint128,
    salt: String,
    slippage: Option<Decimal>,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;

    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let this = env.contract.address;

    let EscherHubStakingLiquidity { exchange_rate, .. } = deps.querier.query_wasm_smart(
        staking_contract.clone(),
        &EscherHubQueryMsg::StakingLiquidity {},
    )?;

    let expected = amount
        .checked_div_floor(exchange_rate)
        .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    // Get the current asset balance in the vault
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset_balance = query_asset_info_balance(&deps.querier, asset_info.clone(), this.clone())?;

    // Validate that we have enough assets to bond
    if asset_balance < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // Create the bond message for the staking contract
    let escher_bond_msg = EscherHubExecuteMsg::Bond {
        slippage,
        expected,
        recipient: None,
        recipient_channel_id: None,
        salt: Some(salt),
    };
    let bond_msg = match asset_info {
        AssetInfo::Token { contract_addr } => WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: staking_contract.to_string(),
                amount,
                msg: to_json_binary(&escher_bond_msg)?,
            })?,
            funds: vec![],
        },
        AssetInfo::NativeToken { .. } => WasmMsg::Execute {
            contract_addr: staking_contract.to_string(),
            msg: to_json_binary(&escher_bond_msg)?,
            funds: Vec::from([Asset {
                info: asset_info,
                amount,
            }
            .as_coin()?]),
        },
    };

    Ok(generate_bond_response(&this, expected, &staking_contract).add_message(bond_msg))
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
