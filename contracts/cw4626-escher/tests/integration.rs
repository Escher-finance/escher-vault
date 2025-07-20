use std::collections::HashMap;

use cosmwasm_std::assert_approx_eq;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Event;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint128;
use cw_multi_test::{App, ContractWrapper, Executor};

use cw4626::{cw20::*, *};
use cw4626_base::contract;
use cw4626_base::msg::*;
use cw4626_base::ContractError;

const USER: &str = "user";
const USER_TWO: &str = "user-two";
const ADMIN: &str = "admin";

const AMOUNT: Uint128 = Uint128::new(1000);
const HALF_FRAC: (Uint128, Uint128) = (Uint128::one(), Uint128::new(2));

fn attrs_to_map(event: &Event) -> HashMap<&str, &str> {
    event
        .attributes
        .iter()
        .map(|a| (a.key.as_str(), a.value.as_str()))
        .collect()
}

fn addr(api: &MockApi, addr: &str) -> Addr {
    api.addr_make(addr)
}

pub fn get_app() -> App {
    App::default()
}

fn instantitate_asset(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    )));
    let decimals = 6;
    let amount = Uint128::from(10000 * 10_u64.pow(decimals as u32));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Token".to_string(),
        symbol: "TKN".to_string(),
        // this is just to be able to airdrop assets at will in the tests
        mint: Some(MinterResponse {
            minter: admin.to_string(),
            cap: None,
        }),
        decimals,
        initial_balances: Vec::from([
            cw20::Cw20Coin {
                amount,
                address: admin.to_string(),
            },
            cw20::Cw20Coin {
                amount,
                address: user.to_string(),
            },
        ]),
        marketing: None,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw20-base-asset", None)
        .unwrap()
}

fn proper_instantiate(app: &mut App, underlying_token_address: Addr) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        contract::execute,
        contract::instantiate,
        contract::query,
    )));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let msg = InstantiateMsg {
        owner: Some(admin.clone()),
        share_name: "Share Token".to_string(),
        share_symbol: "sTKN".to_string(),
        share_marketing: None,
        underlying_token_address,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw4626-base", None)
        .unwrap()
}

#[test]
fn test_todo() {
    assert!(true);
}
