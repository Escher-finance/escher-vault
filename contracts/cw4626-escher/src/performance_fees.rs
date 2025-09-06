use cosmwasm_std::{Deps, DepsMut, Env, Response, StdResult, Uint128, Decimal, MessageInfo, WasmMsg, to_json_binary};

use crate::{
    state::{PERFORMANCE_FEE_CONFIG, PENDING_FEE, TOWER_CONFIG},
    helpers::get_tokens,
    ContractError,
};

/// Calculate and charge performance fees based on asset growth
pub fn calculate_performance_fees(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Only managers can trigger fee calculation
    crate::access_control::only_role(deps.storage, &info.sender, crate::state::AccessControlRole::Manager {})?;
    
    let mut config = PERFORMANCE_FEE_CONFIG.load(deps.storage)?;
    let current_block = env.block.height;
    
    // Check if enough time has passed since last fee calculation
    if current_block < config.last_fee_calculation + config.fee_calculation_interval {
        return Err(ContractError::FeeCalculationTooEarly {
            current_block,
            required_block: config.last_fee_calculation + config.fee_calculation_interval,
        });
    }
    
    // Calculate current total assets
    let this = env.contract.address;
    let tokens = get_tokens(&this, &deps.as_ref())?;
    let current_assets = tokens.total_assets;
    
    // Calculate exchange rate
    let exchange_rate = if tokens.total_shares.is_zero() {
        Decimal::one()
    } else {
        Decimal::from_ratio(tokens.total_assets, tokens.total_shares)
    };
    
    // Only charge fees if exchange rate > 1.0 (prevents fees from deposits)
    if exchange_rate <= Decimal::one() {
        // No profit, just update the last calculation time and assets snapshot
        config.last_fee_calculation = current_block;
        config.last_assets_snapshot = current_assets;
        PERFORMANCE_FEE_CONFIG.save(deps.storage, &config)?;
        
        return Ok(Response::new()
            .add_event(cosmwasm_std::Event::new("performance_fee_calculation")
                .add_attribute("action", "no_profit")
                .add_attribute("current_assets", current_assets.to_string())
                .add_attribute("exchange_rate", exchange_rate.to_string())
                .add_attribute("block_height", current_block.to_string())));
    }
    
    // Calculate asset growth since last fee calculation
    let asset_growth = if current_assets > config.last_assets_snapshot {
        current_assets - config.last_assets_snapshot
    } else {
        Uint128::zero()
    };
    
    // Only charge fees if there's actual growth
    if asset_growth.is_zero() {
        config.last_fee_calculation = current_block;
        config.last_assets_snapshot = current_assets;
        PERFORMANCE_FEE_CONFIG.save(deps.storage, &config)?;
        
        return Ok(Response::new()
            .add_event(cosmwasm_std::Event::new("performance_fee_calculation")
                .add_attribute("action", "no_growth")
                .add_attribute("current_assets", current_assets.to_string())
                .add_attribute("last_assets", config.last_assets_snapshot.to_string())
                .add_attribute("block_height", current_block.to_string())));
    }
    
    // Calculate fee value (10% of growth) in underlying asset terms
    let fee_value = config.fee_rate * Decimal::from_ratio(asset_growth, Uint128::one());
    let fee_value_uint = fee_value.to_uint_floor();
    
    // Only charge fees if they're above a minimum threshold (avoid dust)
    if fee_value_uint < Uint128::new(1000) { // Minimum 1000 units
        config.last_fee_calculation = current_block;
        config.last_assets_snapshot = current_assets;
        PERFORMANCE_FEE_CONFIG.save(deps.storage, &config)?;
        
        return Ok(Response::new()
            .add_event(cosmwasm_std::Event::new("performance_fee_calculation")
                .add_attribute("action", "fee_too_small")
                .add_attribute("fee_value", fee_value_uint.to_string())
                .add_attribute("block_height", current_block.to_string())));
    }
    
    // Calculate fee percentage of total assets
    let fee_percentage = Decimal::from_ratio(fee_value_uint, current_assets);
    
    // For now, we'll create a response that indicates the fee should be distributed
    // In a full implementation, we'd need to:
    // 1. Query the vault's current LP position
    // 2. Query pending incentives
    // 3. Calculate proportional distribution across all assets
    // 4. Send the appropriate tokens to the fee recipient
    
    // This is a simplified approach - in production, you'd want to:
    // - Withdraw from LP position proportionally
    // - Claim incentives proportionally  
    // - Send the actual tokens to the fee recipient
    
    // Update last calculation time and assets snapshot
    config.last_fee_calculation = current_block;
    config.last_assets_snapshot = current_assets;
    PERFORMANCE_FEE_CONFIG.save(deps.storage, &config)?;
    
    // Store the calculated fee for manual distribution
    // This approach is more practical and secure
    let fee_info = crate::state::FeeInfo {
        amount: fee_value_uint,
        percentage: fee_percentage,
        calculated_at: current_block,
        distributed: false,
    };
    crate::state::PENDING_FEE.save(deps.storage, &fee_info)?;
    
    Ok(Response::new()
        .add_event(cosmwasm_std::Event::new("performance_fee_calculation")
            .add_attribute("action", "fee_calculated")
            .add_attribute("current_assets", current_assets.to_string())
            .add_attribute("last_assets", config.last_assets_snapshot.to_string())
            .add_attribute("asset_growth", asset_growth.to_string())
            .add_attribute("exchange_rate", exchange_rate.to_string())
            .add_attribute("fee_rate", config.fee_rate.to_string())
            .add_attribute("fee_value", fee_value_uint.to_string())
            .add_attribute("fee_percentage", fee_percentage.to_string())
            .add_attribute("fee_recipient", config.fee_recipient.to_string())
            .add_attribute("block_height", current_block.to_string())
            .add_attribute("note", "Fee calculated - use DistributeFee to send LP tokens and incentives")))
}

/// Query performance fee configuration
pub fn query_performance_fee_config(deps: Deps) -> StdResult<crate::msg::PerformanceFeeConfigResponse> {
    let config = PERFORMANCE_FEE_CONFIG.load(deps.storage)?;
    Ok(crate::msg::PerformanceFeeConfigResponse {
        fee_rate: config.fee_rate,
        fee_recipient: config.fee_recipient,
        initial_assets: config.initial_assets,
        last_fee_calculation: config.last_fee_calculation,
        fee_calculation_interval: config.fee_calculation_interval,
        last_assets_snapshot: config.last_assets_snapshot,
    })
}

/// Query asset growth information
pub fn query_asset_growth(deps: Deps, env: &Env) -> StdResult<crate::msg::AssetGrowthResponse> {
    let config = PERFORMANCE_FEE_CONFIG.load(deps.storage)?;
    let this = env.contract.address.clone();
    let tokens = get_tokens(&this, &deps)?;
    
    let current_assets = tokens.total_assets;
    let exchange_rate = if tokens.total_shares.is_zero() {
        Decimal::one()
    } else {
        Decimal::from_ratio(tokens.total_assets, tokens.total_shares)
    };
    
    let asset_growth = if current_assets > config.last_assets_snapshot {
        current_assets - config.last_assets_snapshot
    } else {
        Uint128::zero()
    };
    
    let blocks_since_last_fee = env.block.height - config.last_fee_calculation;
    let next_fee_calculation_block = config.last_fee_calculation + config.fee_calculation_interval;
    
    // Can charge fee if: exchange_rate > 1.0 AND asset_growth > 0 AND enough time passed
    let can_charge_fee = exchange_rate > Decimal::one() 
        && !asset_growth.is_zero() 
        && env.block.height >= next_fee_calculation_block;
    
    Ok(crate::msg::AssetGrowthResponse {
        current_assets,
        initial_assets: config.initial_assets,
        last_assets_snapshot: config.last_assets_snapshot,
        asset_growth,
        exchange_rate,
        blocks_since_last_fee,
        next_fee_calculation_block,
        can_charge_fee,
    })
}

/// Distribute pending fee by sending actual assets (LP tokens, incentives, etc.)
pub fn distribute_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Only managers can distribute fees
    crate::access_control::only_role(deps.storage, &info.sender, crate::state::AccessControlRole::Manager {})?;
    
    // Check if there's a pending fee
    let mut fee_info = PENDING_FEE.may_load(deps.storage)?
        .ok_or(ContractError::Std(cosmwasm_std::StdError::not_found("pending_fee")))?;
    
    if fee_info.distributed {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err("Fee already distributed")));
    }
    
    let config = PERFORMANCE_FEE_CONFIG.load(deps.storage)?;
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    let mut messages = vec![];
    
    // 1. Withdraw proportional LP position
    let lp_token_amount = (fee_info.percentage * Decimal::from_ratio(Uint128::new(1000000), Uint128::one())).to_uint_floor();
    let lp_withdraw_msg = WasmMsg::Execute {
        contract_addr: tower_config.lp.to_string(),
        msg: to_json_binary(&serde_json::json!({
            "withdraw_liquidity": {
                "lp_token_amount": lp_token_amount.to_string()
            }
        }))?,
        funds: vec![],
    };
    messages.push(cosmwasm_std::CosmosMsg::Wasm(lp_withdraw_msg));
    
    // 2. Claim proportional incentives
    let claim_incentives_msg = WasmMsg::Execute {
        contract_addr: tower_config.tower_incentives.to_string(),
        msg: to_json_binary(&serde_json::json!({
            "claim_rewards": {}
        }))?,
        funds: vec![],
    };
    messages.push(cosmwasm_std::CosmosMsg::Wasm(claim_incentives_msg));
    
    // 3. Send underlying asset (ubbn) as fee
    let underlying_asset = crate::state::UNDERLYING_ASSET.load(deps.storage)?;
    if let astroport::asset::AssetInfo::NativeToken { denom } = underlying_asset {
        let bank_msg = cosmwasm_std::BankMsg::Send {
            to_address: config.fee_recipient.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom,
                amount: fee_info.amount,
            }],
        };
        messages.push(cosmwasm_std::CosmosMsg::Bank(bank_msg));
    }
    
    // Mark fee as distributed
    fee_info.distributed = true;
    PENDING_FEE.save(deps.storage, &fee_info)?;
    
    Ok(Response::new()
        .add_messages(messages)
        .add_event(cosmwasm_std::Event::new("fee_distribution")
            .add_attribute("action", "fee_distributed")
            .add_attribute("fee_amount", fee_info.amount.to_string())
            .add_attribute("fee_percentage", fee_info.percentage.to_string())
            .add_attribute("fee_recipient", config.fee_recipient.to_string())
            .add_attribute("block_height", env.block.height.to_string())
            .add_attribute("note", "Fee distributed: LP withdrawn, incentives claimed, underlying asset sent")))
}