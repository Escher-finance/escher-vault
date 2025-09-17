use astroport::asset::{Asset, AssetInfo};
use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{
    Addr, Decimal, DepsMut, MessageInfo, StdError, Timestamp, Uint128, WasmMsg, to_json_binary,
};

use crate::{
    ContractError,
    asset::{asset_cw20_send_or_attach_funds, query_asset_info_balance},
    error::ContractResult,
    helpers::validate_salt,
    state::{LST_CONFIG, LstConfig, NonZkgmLstConfig, TowerConfig, UNDERLYING_ASSET},
    zkgm::validate_and_store_zkgm_lst_config,
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

#[cw_serde]
pub struct EscherHubParameters {
    pub underlying_coin_denom: String,
    pub liquidstaking_denom: String,
    pub ucs03_relay_contract: String,
    pub unbonding_time: u64,
    // liquid_staking denom/cw20 contract address
    pub cw20_address: Addr,
    // reward contract address
    pub reward_address: Addr,
    // fee fee_rate
    pub fee_rate: Decimal,
    // fee receiver
    pub fee_receiver: Addr,
    // batch period range in seconds to execute batch
    pub batch_period: u64,
    // minimum bond/stake amount
    pub min_bond: Uint128,
    // minimum unbond/unstake amount
    pub min_unbond: Uint128,
    // limit per batch
    // this is the max number of unbonding records that can be processed in one batch
    pub batch_limit: u32,
    // handler of cw20 staking token transfer, as ucs03 fee payer address and also minted cw20 staking token receiver
    pub transfer_handler: String,
    // ucs03 transfer fee from babylon to other
    pub transfer_fee: Uint128,
    // zkgm token_minter address as cw20 allowance spender
    pub zkgm_token_minter: String,
}

#[cw_serde]
pub struct EscherHubStakingLiquidity {
    pub amount: Uint128,
    pub delegated: Uint128,
    pub reward: Uint128,
    pub unclaimed_reward: Uint128,
    pub exchange_rate: Decimal,
    pub time: Timestamp,
    pub total_supply: Uint128,
    pub adjusted_supply: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum EscherHubQueryMsg {
    #[returns(EscherHubStakingLiquidity)]
    StakingLiquidity {},
    #[returns(EscherHubParameters)]
    Parameters {},
}

#[cw_serde]
pub enum EscherHubExecuteMsg {
    Bond {
        slippage: Option<Decimal>,
        expected: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        salt: Option<String>,
    },
    Unstake {
        amount: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        recipient_ibc_channel_id: Option<u32>,
    },
}

/// Validates and stores a NON-ZKGM LST config
///
/// # Errors
/// Will return error if queries or validation fails
pub fn validate_and_store_non_zkgm_lst_config(
    deps: &mut DepsMut,
    config: &NonZkgmLstConfig,
    other_lp_token: &AssetInfo,
) -> ContractResult<LstConfig> {
    let _ = deps
        .querier
        .query_wasm_smart::<EscherHubStakingLiquidity>(
            config.lst_contract.clone(),
            &EscherHubQueryMsg::StakingLiquidity {},
        )
        .map_err(|_| ContractError::InvalidStakingContract {})?;
    let EscherHubParameters { cw20_address, .. } = deps
        .querier
        .query_wasm_smart::<EscherHubParameters>(
            config.lst_contract.clone(),
            &EscherHubQueryMsg::Parameters {},
        )
        .map_err(|_| ContractError::InvalidStakingContract {})?;
    let cw20_info = AssetInfo::Token { contract_addr: cw20_address };
    if cw20_info != *other_lp_token {
        return Err(ContractError::InvalidStakingContract {});
    }
    let lst_config = LstConfig::NonZkgm(config.clone());
    LST_CONFIG.save(deps.storage, &lst_config)?;
    Ok(lst_config)
}

/// Returns (`bond_msg`, `expected`)
///
/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn internal_bond(
    deps: &mut DepsMut,
    this: &Addr,
    staking_contract: &Addr,
    amount: Uint128,
    salt: String,
    slippage: Option<Decimal>,
) -> ContractResult<(WasmMsg, Uint128)> {
    validate_salt(&salt)?;

    let EscherHubStakingLiquidity { exchange_rate, .. } = deps
        .querier
        .query_wasm_smart(staking_contract.clone(), &EscherHubQueryMsg::StakingLiquidity {})?;

    let expected = amount
        .checked_div_floor(exchange_rate)
        .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    // Get the current asset balance in the vault
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset_balance = query_asset_info_balance(&deps.querier, asset_info.clone(), this)?;

    // Validate that we have enough assets to bond
    if asset_balance < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // Create the bond message for the staking contract
    let bond_msg = asset_cw20_send_or_attach_funds(
        Asset { info: asset_info, amount },
        staking_contract,
        to_json_binary(&EscherHubExecuteMsg::Bond {
            slippage,
            expected,
            recipient: None,
            recipient_channel_id: None,
            salt: Some(salt),
        })?,
    )?;

    Ok((bond_msg, expected))
}

/// Returns (`unbond_msg`, `expected`)
///
/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn internal_unbond(
    deps: &mut DepsMut,
    info: &MessageInfo,
    other_lp_token: AssetInfo,
    staking_contract: &Addr,
    amount: Uint128,
) -> ContractResult<(WasmMsg, Uint128)> {
    // Query the staking contract to get current liquidity info
    let EscherHubStakingLiquidity { exchange_rate, .. } = deps
        .querier
        .query_wasm_smart(staking_contract.clone(), &EscherHubQueryMsg::StakingLiquidity {})?;

    // Calculate the expected amount of underlying tokens to receive
    let expected = amount
        .checked_mul_floor(exchange_rate)
        .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))?;

    // Create the unbond message by sending eBABY tokens to the staking contract
    // The staking contract's Receive handler will process the unbond when it receives the eBABY tokens
    let unbond_msg = asset_cw20_send_or_attach_funds(
        Asset { info: other_lp_token, amount },
        staking_contract,
        to_json_binary(&EscherHubExecuteMsg::Unstake {
            amount,
            recipient: Some(info.sender.to_string()), // Send unstaked tokens back to the caller
            recipient_channel_id: None,
            recipient_ibc_channel_id: None,
        })?,
    )?;

    Ok((unbond_msg, expected))
}
