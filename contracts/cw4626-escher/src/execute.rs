use astroport::{
    asset::{Asset, AssetInfo},
    pair_concentrated::QueryMsg as PairConcentratedQueryMsg,
};
use cosmwasm_std::{
    to_json_binary, Addr, Decimal, Decimal256, DepsMut, Env, MessageInfo, Response,
    Uint128, WasmMsg,
};
use cw4626::cw20;

use crate::{
    access_control::only_role,
    asset_info::query_asset_info_balance,
    helpers::_deposit,
    query,
    responses::generate_bond_response,
    staking::{EscherHubExecuteMsg, EscherHubQueryMsg, EscherHubStakingLiquidity},
    state::{
        AccessControlRole, PricesMap, ACCESS_CONTROL, STAKING_CONTRACT, TOWER_CONFIG,
        UNDERLYING_ASSET,
    },
    tower::{add_tower_liquidity, update_and_validate_prices},
    ContractError,
};

/// Validates amount parameter for security
fn validate_amount(amount: Uint128, _param_name: &str) -> Result<(), ContractError> {
    if amount.is_zero() {
        return Err(ContractError::invalid_amount(amount, "cannot be zero"));
    }
    Ok(())
}

/// Validates salt parameter for security
fn validate_salt(salt: &str) -> Result<(), ContractError> {
    if salt.is_empty() {
        return Err(ContractError::empty_input("salt"));
    }
    if salt.len() > 100 {
        return Err(ContractError::invalid_length("salt", 1, 100, salt.len() as u32));
    }
    // Check for dangerous characters
    let invalid_chars: Vec<char> = salt.chars()
        .filter(|&c| c == '\x00' || c == '<' || c == '>')
        .collect();
    if !invalid_chars.is_empty() {
        return Err(ContractError::invalid_characters("salt", invalid_chars));
    }
    Ok(())
}

/// Validates slippage tolerance for security
fn validate_slippage(slippage: Option<Decimal>) -> Result<(), ContractError> {
    if let Some(slippage) = slippage {
        if slippage > Decimal::percent(50) || slippage < Decimal::percent(1) {
            return Err(ContractError::invalid_slippage(
                slippage,
                Decimal::percent(1),
                Decimal::percent(50)
            ));
        }
    }
    Ok(())
}

/// Validates receiver address for security
fn validate_receiver(receiver: &Addr) -> Result<(), ContractError> {
    if receiver.as_str().is_empty() {
        return Err(ContractError::empty_input("receiver"));
    }
    Ok(())
}

pub fn add_to_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    
    // Validate the address being added
    crate::access_control::validate_address(&address)?;
    
    let address_str = address.to_string();
    
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        let mut addrs = addrs.unwrap_or_default();
        
        // Check if address already exists
        if addrs.contains(&address) {
            return Err(ContractError::validation_error(
                "address",
                &format!("Address already has {} role", role)
            ));
        }
        
        // Check role size limit
        if addrs.len() >= 20 {
            return Err(ContractError::validation_error(
                "role_size",
                "Role size limit exceeded: max 20 addresses allowed"
            ));
        }
        
        addrs.push(address);
        Ok(addrs)
    })?;
    
    Ok(Response::new().add_attribute("action", "add_to_role")
        .add_attribute("role", role.to_string())
        .add_attribute("address", address_str))
}

pub fn remove_from_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    
    // Validate the address being removed
    crate::access_control::validate_address(&address)?;
    
    let address_str = address.to_string();
    
    // Prevent removing the last manager to avoid permanent lockout
    if matches!(role, AccessControlRole::Manager {}) {
        let current_managers = ACCESS_CONTROL.load(deps.storage, role.key())?;
        if current_managers.len() <= 1 {
            return Err(ContractError::security_error(
                "Cannot remove the last manager to prevent permanent lockout"
            ));
        }
    }
    
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        let addrs = addrs.unwrap_or_default();
        let original_len = addrs.len();
        let filtered_addrs: Vec<_> = addrs.into_iter().filter(|a| a != address).collect();
        
        // Check if the address was actually in the role
        if filtered_addrs.len() == original_len {
            return Err(ContractError::validation_error(
                "address",
                &format!("Address does not have {} role", role)
            ));
        }
        
        Ok(filtered_addrs)
    })?;
    
    Ok(Response::new().add_attribute("action", "remove_from_role")
        .add_attribute("role", role.to_string())
        .add_attribute("address", address_str))
}

pub fn oracle_update_prices(
    deps: DepsMut,
    sender: Addr,
    prices: PricesMap,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Oracle {})?;
    
    // CRITICAL: Validate prices map
    if prices.is_empty() {
        return Err(ContractError::validation_error(
            "prices",
            "Prices map cannot be empty"
        ));
    }
    
    // Validate each price is positive
    for (token, price) in &prices {
        if price.is_zero() {
            return Err(ContractError::validation_error(
                "price",
                "Price for token cannot be zero"
            ));
        }
        if token.is_empty() {
            return Err(ContractError::empty_input("token_identifier"));
        }
    }
    
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
    
    // CRITICAL: Validate all input parameters
    validate_amount(amount, "amount")?;
    validate_salt(&salt)?;
    validate_slippage(slippage)?;

    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let this = env.contract.address;

    let EscherHubStakingLiquidity { exchange_rate, .. } = deps.querier.query_wasm_smart(
        staking_contract.clone(),
        &EscherHubQueryMsg::StakingLiquidity {},
    )?;

    let expected = amount
        .checked_div_floor(exchange_rate)
        .map_err(|err| ContractError::math_error("division", &err.to_string()))?;

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
    // CRITICAL: Validate input parameters
    validate_amount(assets, "assets")?;
    validate_receiver(&receiver)?;
    
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
    // CRITICAL: Validate input parameters
    validate_amount(shares, "shares")?;
    validate_receiver(&receiver)?;
    
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

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    underlying_token_amount: Uint128,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    
    // CRITICAL: Validate input parameters
    validate_amount(underlying_token_amount, "underlying_token_amount")?;

    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let lp_price = Decimal::try_from(deps.querier.query_wasm_smart::<Decimal256>(
        tower_config.lp.clone(),
        &PairConcentratedQueryMsg::LpPrice {},
    )?)
    .map_err(|err| ContractError::math_error("decimal_conversion", &err.to_string()))?;

    let other_lp_token_amount = if tower_config.is_underlying_first_lp_asset {
        underlying_token_amount.checked_div_floor(lp_price)
    } else {
        underlying_token_amount.checked_mul_floor(lp_price)
    }
    .map_err(|err| ContractError::math_error("lp_calculation", &err.to_string()))?;

    let this = env.contract.address;
    let underlying_balance = query_asset_info_balance(
        &deps.querier,
        tower_config.lp_underlying_asset.clone(),
        this.clone(),
    )?;
    let other_lp_balance =
        query_asset_info_balance(&deps.querier, tower_config.lp_other_asset.clone(), this)?;

    if underlying_token_amount.is_zero()
        || other_lp_token_amount.is_zero()
        || underlying_balance < underlying_token_amount
        || other_lp_balance < other_lp_token_amount
    {
        return Err(ContractError::InsufficientFunds {});
    }

    let msg = add_tower_liquidity(
        &tower_config,
        underlying_token_amount,
        other_lp_token_amount,
    )?;

    Ok(Response::new().add_message(msg))
}
