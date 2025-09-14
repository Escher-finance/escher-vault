use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Decimal, Event, Response, Timestamp, Uint128};

pub const EVENT_BOND: &str = "bond";
pub const EVENT_UNBOND: &str = "unbond";
pub const EVENT_DEPOSIT: &str = "deposit";
pub const EVENT_WITHDRAW: &str = "withdraw";
pub const EVENT_ADD_LIQUIDITY: &str = "add_liquidity";
pub const EVENT_REMOVE_LIQUIDITY: &str = "remove_liquidity";
pub const EVENT_CLAIM_INCENTIVES: &str = "claim_incentives";
pub const EVENT_SWAP: &str = "swap";
pub const EVENT_ADD_ROLE: &str = "add_role";
pub const EVENT_REMOVE_ROLE: &str = "remove_role";
pub const EVENT_ORACLE_UPDATE_PRICES: &str = "oracle_update_prices";
pub const EVENT_REQUEST_REDEMPTION: &str = "request_redemption";
pub const EVENT_COMPLETE_REDEMPTION: &str = "complete_redemption";

#[must_use]
pub fn generate_withdraw_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_WITHDRAW)
            .add_attribute("withdrawer", caller)
            .add_attribute("receiver", receiver)
            .add_attribute("assets_received", assets)
            .add_attribute("shares_burned", shares),
    )
}

#[must_use]
pub fn generate_bond_response(
    sender: &Addr,
    amount: Uint128,
    expected: Uint128,
    staking_contract: &Addr,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_BOND)
            .add_attribute("sender", sender)
            .add_attribute("amount", amount)
            .add_attribute("expected", expected)
            .add_attribute("staking_contract", staking_contract),
    )
}

#[must_use]
pub fn generate_unbond_response(
    sender: &Addr,
    expected: Uint128,
    staking_contract: &Addr,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_UNBOND)
            .add_attribute("sender", sender)
            .add_attribute("expected", expected)
            .add_attribute("staking_contract", staking_contract),
    )
}

#[must_use]
pub fn generate_deposit_response(
    sender: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_DEPOSIT)
            .add_attribute("depositor", sender)
            .add_attribute("receiver", receiver)
            .add_attribute("assets_transferred", assets)
            .add_attribute("shares_minted", shares),
    )
}

#[must_use]
pub fn generate_deposit_with_fee_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    user_shares: Uint128,
    fee_shares: Uint128,
    entry_fee_rate: Decimal,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_DEPOSIT)
            .add_attribute("depositor", caller)
            .add_attribute("receiver", receiver)
            .add_attribute("assets_transferred", assets)
            .add_attribute("user_shares_minted", user_shares)
            .add_attribute("fee_shares_minted", fee_shares)
            .add_attribute("entry_fee_rate", entry_fee_rate.to_string()),
    )
}

#[must_use]
pub fn add_liquidity_event(
    sender: &Addr,
    underlying_token_amount: Uint128,
    other_lp_token_amount: Uint128,
    lp_contract: &Addr,
) -> Event {
    Event::new(EVENT_ADD_LIQUIDITY)
        .add_attribute("sender", sender)
        .add_attribute("underlying_token_amount", underlying_token_amount)
        .add_attribute("other_lp_token_amount", other_lp_token_amount)
        .add_attribute("lp_contract", lp_contract)
}

#[must_use]
pub fn remove_liquidity_event(
    sender: &Addr,
    lp_token_amount: Uint128,
    lp_contract: &Addr,
) -> Event {
    Event::new(EVENT_REMOVE_LIQUIDITY)
        .add_attribute("sender", sender)
        .add_attribute("lp_token_amount", lp_token_amount)
        .add_attribute("lp_contract", lp_contract)
}

#[must_use]
pub fn claim_incentives_event(sender: &Addr, lp_contract: &Addr) -> Event {
    Event::new(EVENT_CLAIM_INCENTIVES)
        .add_attribute("sender", sender)
        .add_attribute("lp_contract", lp_contract)
}

#[must_use]
pub fn swap_event(sender: &str, amount: Uint128, asset_info: &AssetInfo) -> Event {
    Event::new(EVENT_SWAP)
        .add_attribute("sender", sender)
        .add_attribute("amount", amount)
        .add_attribute("asset_info", asset_info.to_string())
}

#[must_use]
pub fn generate_add_role_response(sender: &str, role: &str, address: &str) -> Response {
    Response::new().add_event(
        Event::new(EVENT_ADD_ROLE)
            .add_attribute("sender", sender)
            .add_attribute("role", role)
            .add_attribute("address", address),
    )
}

#[must_use]
pub fn generate_remove_role_response(sender: &str, role: &str, address: &str) -> Response {
    Response::new().add_event(
        Event::new(EVENT_REMOVE_ROLE)
            .add_attribute("sender", sender)
            .add_attribute("role", role)
            .add_attribute("address", address),
    )
}

#[must_use]
pub fn generate_request_redemption_response(
    redemption_id: u64,
    owner: &Addr,
    receiver: &Addr,
    shares_locked: Uint128,
    created_at: Timestamp,
    expected_assets: &[Asset],
) -> Response {
    // Format expected assets as a comma-separated string
    let expected_assets_str = expected_assets
        .iter()
        .map(|a| format!("{}={}", a.info, a.amount))
        .collect::<Vec<_>>()
        .join(",");

    let e = Event::new(EVENT_REQUEST_REDEMPTION)
        .add_attribute("redemption_id", redemption_id.to_string())
        .add_attribute("owner", owner)
        .add_attribute("receiver", receiver)
        .add_attribute("shares_locked", shares_locked)
        .add_attribute("expected_assets_count", expected_assets.len().to_string())
        .add_attribute("expected_assets", expected_assets_str)
        .add_attribute("created_at", created_at.to_string())
        .add_attribute(
            "total_expected_value",
            expected_assets.iter().map(|a| a.amount).sum::<Uint128>(),
        );

    Response::new().add_event(e)
}

#[must_use]
pub fn generate_complete_redemption_response(
    redemption_id: u64,
    receiver: &Addr,
    shares_burned: Uint128,
    completed_at: Timestamp,
    tx_hash: &str,
    distributed_assets: &[Asset],
) -> Response {
    // Format distributed assets as a comma-separated string
    let distributed_assets_str = distributed_assets
        .iter()
        .map(|a| format!("{}={}", a.info, a.amount))
        .collect::<Vec<_>>()
        .join(",");
    let e = Event::new(EVENT_COMPLETE_REDEMPTION)
        .add_attribute("redemption_id", redemption_id.to_string())
        .add_attribute("receiver", receiver)
        .add_attribute("shares_burned", shares_burned)
        .add_attribute("completed_at", completed_at.to_string())
        .add_attribute("tx_hash", tx_hash)
        .add_attribute("distributed_assets", distributed_assets_str)
        .add_attribute("distributed_assets_count", distributed_assets.len().to_string());

    Response::new().add_event(e)
}

#[must_use]
pub fn generate_oracle_update_prices_response(
    sender: &str,
    prices: &crate::state::PricesMap,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_ORACLE_UPDATE_PRICES).add_attribute("sender", sender).add_attribute(
            "prices",
            prices.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(","),
        ),
    )
}
