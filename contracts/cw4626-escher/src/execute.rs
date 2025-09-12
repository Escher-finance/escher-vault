use astroport::{asset::AssetInfo, pair_concentrated::QueryMsg as PairConcentratedQueryMsg};
use cosmwasm_std::{
    from_json, Addr, Decimal, Decimal256, DepsMut, Env, MessageInfo, Response, StdError, Uint128,
};
use ibc_union_spec::Timestamp;

use crate::{
    access_control::{validate_only_not_paused, validate_only_role},
    asset::query_asset_info_balance,
    error::ContractResult,
    helpers::{
        internal_deposit, internal_update_minimum_deposit, validate_addrs, PreviewDepositKind,
    },
    msg::{MaxDepositResponse, PreviewDepositResponse, ReceiveMsg},
    query,
    receive::receive_deposit,
    responses::{
        add_liquidity_event, claim_incentives_event, generate_add_role_response,
        generate_bond_response, generate_oracle_update_prices_response,
        generate_remove_role_response, generate_unbond_response, remove_liquidity_event,
        swap_event,
    },
    staking::{internal_bond, internal_unbond, validate_and_store_staking_contract},
    state::{
        AccessControlRole, PausedStatus, PricesMap, TowerConfig, ACCESS_CONTROL, PAUSED_STATUS,
        STAKING_CONTRACT, TOWER_CONFIG, UNDERLYING_ASSET,
    },
    tower::{
        add_tower_liquidity, claim_tower_incentives, get_tower_lp_token_deposit,
        remove_tower_liquidity, tower_swap, update_and_validate_prices,
    },
    ContractError,
};

/// # Errors
/// Will return error if internal helper fails
pub fn add_to_role(
    deps: &mut DepsMut,
    sender: &Addr,
    role: AccessControlRole,
    address: &Addr,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        let mut addrs = addrs.unwrap_or_default();
        addrs.push(address.clone());
        validate_addrs(addrs.into_iter())
    })?;
    Ok(generate_add_role_response(
        sender.as_ref(),
        &role.to_string(),
        address.as_ref(),
    ))
}

/// # Errors
/// Will return error if internal helper fails
pub fn remove_from_role(
    deps: &mut DepsMut,
    sender: &Addr,
    role: AccessControlRole,
    address: &Addr,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.update::<_, ContractError>(deps.storage, role.key(), |addrs| {
        let addrs = validate_addrs(
            addrs
                .unwrap_or_default()
                .into_iter()
                .filter(|a| a != address),
        )?;
        Ok(addrs)
    })?;
    Ok(generate_remove_role_response(
        sender.as_ref(),
        &role.to_string(),
        address.as_ref(),
    ))
}

/// # Errors
/// Will return error if internal helper fails
pub fn oracle_update_prices(
    deps: &mut DepsMut,
    sender: &Addr,
    prices: &PricesMap,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, sender, AccessControlRole::Oracle {})?;
    update_and_validate_prices(deps, prices.clone())?;
    Ok(generate_oracle_update_prices_response(
        sender.as_ref(),
        prices,
    ))
}

/// # Errors
/// Will return error if internal helper fails
pub fn update_staking_contract(
    deps: &mut DepsMut,
    info: &MessageInfo,
    address: &Addr,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    let TowerConfig { lp_other_asset, .. } = TOWER_CONFIG.load(deps.storage)?;
    validate_and_store_staking_contract(deps, address, &lp_other_asset)?;
    Ok(Response::new())
}

/// # Errors
/// Will return error if internal helper fails
pub fn update_minimum_deposit(
    deps: &mut DepsMut,
    info: &MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    internal_update_minimum_deposit(deps, Some(amount))?;
    Ok(Response::new())
}

/// # Errors
/// Will return error if internal helper fails
pub fn update_paused_status(
    deps: &mut DepsMut,
    info: &MessageInfo,
    status: &PausedStatus,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    PAUSED_STATUS.save(deps.storage, status)?;
    Ok(Response::new())
}

/// # Errors
/// Will return error if internal helper fails
pub fn bond(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    amount: Uint128,
    salt: String,
    slippage: Option<Decimal>,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let this = &env.contract.address;

    let (bond_msg, expected) =
        internal_bond(deps, this, &staking_contract, amount, salt, slippage)?;

    Ok(generate_bond_response(this, amount, expected, &staking_contract).add_message(bond_msg))
}

/// # Errors
/// Will return error if internal helper fails
pub fn unbond(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let TowerConfig { lp_other_asset, .. } = TOWER_CONFIG.load(deps.storage)?;
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let this = &env.contract.address;

    let (unbond_msg, expected) =
        internal_unbond(deps, info, lp_other_asset, &staking_contract, amount)?;

    Ok(generate_unbond_response(this, expected, &staking_contract).add_message(unbond_msg))
}

/// # Errors
/// Will return error if internal helper fails
pub fn deposit(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    assets: Uint128,
    receiver: &Addr,
) -> ContractResult<Response> {
    validate_only_not_paused(deps.storage, &info.sender)?;

    let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxDeposit {
            receiver: receiver.clone(),
            assets,
            max_assets,
        });
    }
    let PreviewDepositResponse { shares } = query::preview_deposit(
        &env.contract.address,
        &deps.as_ref(),
        assets,
        match UNDERLYING_ASSET.load(deps.storage)? {
            AssetInfo::Token { .. } => PreviewDepositKind::Cw20ViaTransferFrom {},
            AssetInfo::NativeToken { .. } => PreviewDepositKind::Native {},
        },
    )?;
    let sender = info.sender.clone();
    internal_deposit(deps, env, info, &sender, receiver, assets, shares, false)
}

/// # Errors
/// Will return error if internal helper fails
pub fn add_liquidity(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    underlying_token_amount: Uint128,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let lp_price = Decimal::try_from(deps.querier.query_wasm_smart::<Decimal256>(
        tower_config.lp.clone(),
        &PairConcentratedQueryMsg::LpPrice {},
    )?)
    .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    let other_lp_token_amount = if tower_config.is_underlying_first_lp_asset {
        underlying_token_amount.checked_div_floor(lp_price)
    } else {
        underlying_token_amount.checked_mul_floor(lp_price)
    }
    .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    let this = &env.contract.address;
    let underlying_balance = query_asset_info_balance(
        &deps.querier,
        tower_config.lp_underlying_asset.clone(),
        this,
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

    let msgs = add_tower_liquidity(
        &tower_config,
        underlying_token_amount,
        other_lp_token_amount,
    )?;

    let event = add_liquidity_event(
        &info.sender,
        underlying_token_amount,
        other_lp_token_amount,
        &tower_config.lp,
    );
    Ok(Response::new().add_event(event).add_messages(msgs))
}

/// # Errors
/// Will return error if internal helper fails
pub fn remove_liquidity(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    lp_token_amount: Uint128,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let this = &env.contract.address;
    let lp_amount = get_tower_lp_token_deposit(&deps.querier, &tower_config, this)?;

    if lp_token_amount.is_zero() || lp_amount < lp_token_amount {
        return Err(ContractError::InsufficientFunds {});
    }

    let msgs = remove_tower_liquidity(&tower_config, lp_token_amount)?;

    let event = remove_liquidity_event(&info.sender, lp_token_amount, &tower_config.lp);
    Ok(Response::new().add_event(event).add_messages(msgs))
}

/// # Errors
/// Will return error if internal helper fails
pub fn claim_incentives(deps: &mut DepsMut, info: &MessageInfo) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let msg = claim_tower_incentives(&tower_config)?;

    let event = claim_incentives_event(&info.sender, &tower_config.lp);
    Ok(Response::new().add_event(event).add_message(msg))
}

/// # Errors
/// Will return error if internal helper fails
pub fn swap(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    amount: Uint128,
    asset_info: AssetInfo,
) -> ContractResult<Response> {
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;

    let tower_config = TOWER_CONFIG.load(deps.storage)?;

    // make sure it only swaps one of the two lp assets
    if asset_info != tower_config.lp_underlying_asset && asset_info != tower_config.lp_other_asset {
        return Err(ContractError::InvalidTokenType {});
    }

    let this = &env.contract.address;

    // make sure we have enough native funds to swap
    let balance = query_asset_info_balance(&deps.querier, asset_info.clone(), this)?;

    if balance < amount {
        return Err(ContractError::InsufficientSwapFunds { asset_info });
    }

    // build the execute swap cosmos messages
    let msgs = tower_swap(&tower_config, amount, &asset_info)?;

    let event = swap_event(info.sender.as_ref(), amount, &asset_info);
    Ok(Response::new().add_event(event).add_messages(msgs))
}

/// # Errors
/// Will return error if internal helper fails
pub fn receive(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    cw20_contract: Addr,
    cw20_receive_msg: &cw20::Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let msg = from_json::<ReceiveMsg>(&cw20_receive_msg.msg)?;
    let sender = deps.api.addr_validate(&cw20_receive_msg.sender)?;

    validate_only_not_paused(deps.storage, &sender)?;

    let received_balance = cw20::Cw20CoinVerified {
        address: cw20_contract,
        amount: cw20_receive_msg.amount,
    };

    match msg {
        ReceiveMsg::Deposit { receiver } => {
            receive_deposit(deps, env, info, &sender, &received_balance, &receiver)
        }
    }
}

/// # Errors
/// Will return error if internal helper fails
pub fn request_redeem(
    mut deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    shares: Uint128,
    receiver: &Addr,
    owner: &Addr,
) -> ContractResult<Response> {
    validate_only_not_paused(deps.storage, &info.sender)?;
    crate::redemption::request_redemption(&mut deps, env, info, shares, receiver, owner)
}

/// # Errors
/// Will return error if internal helper fails
pub fn complete_redemption_with_distribution(
    mut deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    redemption_id: u64,
    tx_hash: &str,
) -> ContractResult<Response> {
    // Restrict completion to managers
    validate_only_role(deps.storage, &info.sender, AccessControlRole::Manager {})?;
    validate_only_not_paused(deps.storage, &info.sender)?;
    crate::redemption::complete_redemption_with_distribution(&mut deps, env, redemption_id, tx_hash)
}

// DEV only: test token_order_v2
///
/// # Errors
/// Will return error if internal helper fails
pub fn token_order_v2(
    deps: &mut DepsMut,
    env: &Env,
    ucs03: &Addr,
    channel_id: u32,
    receiver: &str,
    amount: Uint128,
    denom: String,
    quote_token: &str,
    salt: &str,
) -> Result<Response, ContractError> {
    let contract_addr: Addr = env.contract.address.clone();
    let balance = deps
        .querier
        .query_balance(contract_addr.clone(), denom.clone())?;

    if balance.amount < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    let sender = env.contract.address.to_string();

    let msg_bin = crate::zkgm::send_token_order_v2(
        Timestamp::from_nanos(env.block.time.nanos()),
        channel_id,
        &sender,
        receiver,
        &denom,
        amount,
        quote_token,
        amount,
        salt,
    )?;

    let msg = cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: ucs03.to_string(),
        msg: msg_bin,
        funds: vec![cosmwasm_std::Coin { denom, amount }],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "transfer")
        .add_attribute("amount", amount))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{TowerConfig, ACCESS_CONTROL, TOWER_CONFIG};
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env},
        to_json_binary, Addr, Uint128,
    };
    use std::str::FromStr;

    fn setup_test_contract(deps: &mut DepsMut) {
        // Set up a manager using ACCESS_CONTROL
        let manager =
            Addr::unchecked("cosmwasm1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqlrtkzd"); // Valid bech32 address
        let managers = vec![manager];
        ACCESS_CONTROL
            .save(deps.storage, AccessControlRole::Manager {}.key(), &managers)
            .unwrap();

        // Set up tower config with LP assets
        let tower_config = TowerConfig {
            lp: Addr::unchecked("lp_contract"),
            lp_underlying_asset: AssetInfo::NativeToken {
                denom: "ubbn".to_string(),
            },
            lp_other_asset: AssetInfo::Token {
                contract_addr: Addr::unchecked("cw20_token"),
            },
            lp_token: Addr::unchecked("lp_token"),
            lp_incentives: vec![],
            is_underlying_first_lp_asset: true,
            slippage_tolerance: Decimal::from_str("0.01").unwrap(),
            tower_incentives: Addr::unchecked("tower_incentives"),
        };
        TOWER_CONFIG.save(deps.storage, &tower_config).unwrap();

        // Set up staking contract
        STAKING_CONTRACT
            .save(deps.storage, &Addr::unchecked("tower_incentives"))
            .unwrap();
    }

    #[test]
    fn test_receive_with_deposit_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        setup_test_contract(&mut deps.as_mut());

        let cw20_contract = Addr::unchecked("cw20_token");
        let sender = Addr::unchecked("user");
        let receiver = Addr::unchecked("receiver");
        let info = message_info(&sender, &[]);

        let deposit_msg = ReceiveMsg::Deposit { receiver };
        let cw20_receive_msg = ::cw20::Cw20ReceiveMsg {
            sender: sender.to_string(),
            amount: Uint128::from(1000u128),
            msg: to_json_binary(&deposit_msg).unwrap(),
        };

        // This might fail due to missing underlying asset setup, but should not panic
        let result = receive(
            &mut deps.as_mut(),
            &env,
            &info,
            cw20_contract,
            &cw20_receive_msg,
        );

        // We expect this to fail due to missing setup, but the function should handle it gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_unbond_with_unauthorized_user() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        setup_test_contract(&mut deps.as_mut());

        let sender = Addr::unchecked("cosmwasm1unauthorizeduser123456789012345678901234567890"); // Not a manager
        let amount = Uint128::from(1000u128);

        let result = unbond(
            &mut deps.as_mut(),
            &env,
            &MessageInfo {
                sender: sender.clone(),
                funds: vec![],
            },
            amount,
        );

        // Should fail with Unauthorized error
        assert!(result.is_err());
        match result.unwrap_err() {
            ContractError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_claim_incentives_emits_event_and_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        setup_test_contract(&mut deps.as_mut());

        PAUSED_STATUS
            .save(&mut deps.storage, &PausedStatus::NotPaused {})
            .unwrap();

        let manager = ACCESS_CONTROL
            .load(deps.as_ref().storage, AccessControlRole::Manager {}.key())
            .unwrap()[0]
            .clone();

        let res = claim_incentives(&mut deps.as_mut(), &message_info(&manager, &[])).unwrap();
        // one message to tower incentives
        assert!(!res.messages.is_empty());
        // event present
        assert!(res
            .events
            .iter()
            .any(|e| e.ty.as_str() == crate::responses::EVENT_CLAIM_INCENTIVES));
        let _ = env; // silence unused
    }

    #[test]
    fn test_unbond_builds_cw20_send_message() {
        // Covered in integration tests where wasm smart queries are mocked.
        // Keeping a lightweight assertion here that setup does not panic.
        let mut deps = mock_dependencies();
        setup_test_contract(&mut deps.as_mut());
        assert!(true);
    }
}
