use cosmwasm_std::{Addr, Response, Uint128};

pub fn generate_withdraw_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("withdrawer", caller)
        .add_attribute("receiver", receiver)
        .add_attribute("assets_received", assets)
        .add_attribute("shares_burned", shares)
}

pub fn generate_bond_response(
    sender: &Addr,
    expected: Uint128,
    staking_contract: &Addr,
) -> Response {
    Response::new()
        .add_attribute("action", "bond")
        .add_attribute("sender", sender)
        .add_attribute("expected", expected)
        .add_attribute("staking_contract", staking_contract)
}
