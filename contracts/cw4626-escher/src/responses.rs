use astroport::asset::AssetInfo;
use cosmwasm_std::{Addr, Event, Response, Uint128};

const EVENT_BOND: &str = "bond";
const EVENT_UNBOND: &str = "unbond";
const EVENT_DEPOSIT: &str = "deposit";
const EVENT_WITHDRAW: &str = "withdraw";
const EVENT_ADD_LIQUIDITY: &str = "add_liquidity";
const EVENT_SWAP: &str = "swap";
const EVENT_ADD_ROLE: &str = "add_role";
const EVENT_REMOVE_ROLE: &str = "remove_role";
const EVENT_ORACLE_UPDATE_PRICES: &str = "oracle_update_prices";

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

pub fn generate_bond_response(
    sender: &Addr,
    expected: Uint128,
    staking_contract: &Addr,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_BOND)
            .add_attribute("sender", sender)
            .add_attribute("expected", expected)
            .add_attribute("staking_contract", staking_contract),
    )
}

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

pub fn swap_event(sender: &str, amount: Uint128, asset_info: &AssetInfo) -> Event {
    Event::new(EVENT_SWAP)
        .add_attribute("sender", sender)
        .add_attribute("amount", amount)
        .add_attribute("asset_info", asset_info.to_string())
}

pub fn generate_add_role_response(sender: &str, role: &str, address: &str) -> Response {
    Response::new().add_event(
        Event::new(EVENT_ADD_ROLE)
            .add_attribute("sender", sender)
            .add_attribute("role", role)
            .add_attribute("address", address),
    )
}

pub fn generate_remove_role_response(sender: &str, role: &str, address: &str) -> Response {
    Response::new().add_event(
        Event::new(EVENT_REMOVE_ROLE)
            .add_attribute("sender", sender)
            .add_attribute("role", role)
            .add_attribute("address", address),
    )
}

pub fn generate_oracle_update_prices_response(
    sender: &str,
    prices: &crate::state::PricesMap,
) -> Response {
    Response::new().add_event(
        Event::new(EVENT_ORACLE_UPDATE_PRICES)
            .add_attribute("sender", sender)
            .add_attribute(
                "prices",
                prices
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
    )
}
