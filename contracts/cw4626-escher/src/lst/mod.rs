pub mod non_zkgm;
pub mod zkgm;

use cosmwasm_std::{DepsMut, Env, Response};

use crate::{
    ContractError,
    error::ContractResult,
    lst::{
        non_zkgm::{internal_non_zkgm_bond, validate_and_store_non_zkgm_lst_config},
        zkgm::{
            get_or_update_this_proxy, internal_zkgm_bond, update_this_proxy,
            validate_and_store_zkgm_lst_config,
        },
    },
    msg::ExecuteBondPayload,
    responses::generate_bond_response,
    state::{LST_CONFIG, LstConfig, PAUSED_STATUS, PausedStatus, TOWER_CONFIG, TowerConfig},
};

/// Validates and stores a new LST config
///
/// # Errors
/// Will return error if validation or state update fails
pub fn validate_and_store_lst_config(
    deps: &mut DepsMut,
    lst_config: &LstConfig,
    tower_config: &TowerConfig,
) -> ContractResult<LstConfig> {
    match lst_config {
        LstConfig::NonZkgm(non_zkgm_lst_config) => validate_and_store_non_zkgm_lst_config(
            deps,
            non_zkgm_lst_config,
            &tower_config.lp_other_asset,
        ),
        LstConfig::Zkgm(zkgm_lst_config) => {
            validate_and_store_zkgm_lst_config(deps, zkgm_lst_config, tower_config)
        }
    }
}

/// Internal update lst config which also updates this proxy
///
/// # Errors
/// Will return error if state queries or updates fail
pub fn internal_update_lst_config(
    deps: &mut DepsMut,
    env: &Env,
    config: &LstConfig,
) -> ContractResult<()> {
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let lst_config = validate_and_store_lst_config(deps, config, &tower_config)?;
    if let LstConfig::Zkgm(zkgm_lst_config) = lst_config {
        update_this_proxy(deps, env, &zkgm_lst_config)?;
    }
    Ok(())
}

/// Internal bond function
///
/// # Errors
/// Will return error if validation or internal helper fails
pub fn internal_bond(
    deps: &mut DepsMut,
    env: &Env,
    bond_payload: &ExecuteBondPayload,
) -> ContractResult<Response> {
    // NOTE: Paused due to bonding; must be unpaused manually for now
    PAUSED_STATUS.save(deps.storage, &PausedStatus::PausedOngoingBonding {})?;

    let lst_config = LST_CONFIG.load(deps.storage)?;
    let this = env.contract.address.clone();

    let res = match lst_config {
        LstConfig::Zkgm(zkgm_lst_config) => {
            let ExecuteBondPayload::Zkgm { amount, salt, min_mint_amount } = bond_payload else {
                return Err(ContractError::PayloadNotMatchingLstConfig {});
            };
            let this_proxy = get_or_update_this_proxy(deps, env, &zkgm_lst_config)?;
            let time = ibc_union_spec::Timestamp::from_nanos(env.block.time.nanos());
            let bond_msg = internal_zkgm_bond(
                &this,
                &this_proxy,
                *amount,
                *min_mint_amount,
                &zkgm_lst_config,
                time,
                salt,
            )?;

            generate_bond_response(
                &this,
                *amount,
                *min_mint_amount,
                &zkgm_lst_config.lst_hub_contract,
            )
            .add_message(bond_msg)
        }
        LstConfig::NonZkgm(non_zkgm_lst_config) => {
            let ExecuteBondPayload::NonZkgm { amount, salt, slippage } = bond_payload else {
                return Err(ContractError::PayloadNotMatchingLstConfig {});
            };
            let staking_contract = non_zkgm_lst_config.lst_contract;
            let (bond_msg, expected) = internal_non_zkgm_bond(
                deps,
                &this,
                &staking_contract,
                *amount,
                salt.clone(),
                *slippage,
            )?;

            generate_bond_response(&this, *amount, expected, staking_contract.as_str())
                .add_message(bond_msg)
        }
    };

    Ok(res)
}
