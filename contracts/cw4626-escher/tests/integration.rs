use std::collections::HashMap;

use astroport::asset::AssetInfo;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Empty;
use cosmwasm_std::Env;
use cosmwasm_std::Event;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cw_multi_test::{App, ContractWrapper, Executor};

use cw4626::{cw20::*, *};
use cw4626_escher::contract;
use cw4626_escher::msg::*;

const USER: &str = "user";
const USER_TWO: &str = "user-two";
const ADMIN: &str = "admin";
const ORACLE: &str = "oracle";

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

mod staking_mock {
    use cosmwasm_std::to_json_binary;
    use cw4626_escher::staking::{EscherHubQueryMsg, EscherHubStakingLiquidity};

    use super::*;
    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(_deps: Deps, _env: Env, msg: EscherHubQueryMsg) -> StdResult<Binary> {
        match msg {
            EscherHubQueryMsg::StakingLiquidity {} => to_json_binary(&EscherHubStakingLiquidity {
                exchange_rate: Decimal::one(),
                ..Default::default()
            }),
        }
    }
}

fn instantiate_staking(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        staking_mock::execute,
        staking_mock::instantiate,
        staking_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(code, addr(api, ADMIN), &Empty {}, &[], "staking", None)
        .unwrap()
}

mod lp_mock {
    use astroport::{asset::PairInfo, pair_concentrated};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::to_json_binary;
    use cw_multi_test::IntoAddr;
    use cw_storage_plus::Item;

    use super::*;

    const A: Item<AssetInfo> = Item::new("asset");

    #[cw_serde]
    pub struct InstantiateMsg {
        pub underlying: Addr,
    }

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        A.save(
            deps.storage,
            &AssetInfo::Token {
                contract_addr: msg.underlying,
            },
        )?;
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(deps: Deps, _env: Env, msg: pair_concentrated::QueryMsg) -> StdResult<Binary> {
        match msg {
            pair_concentrated::QueryMsg::Pair {} => {
                let a = A.load(deps.storage)?;
                to_json_binary(&PairInfo {
                    asset_infos: Vec::from([
                        a,
                        AssetInfo::NativeToken {
                            denom: "foo".to_string(),
                        },
                    ]),
                    contract_addr: "pair".into_addr(),
                    liquidity_token: "liqtoken".to_string(),
                    pair_type: astroport::factory::PairType::Concentrated {},
                })
            }
            _ => unimplemented!(),
        }
    }
}

fn instantiate_lp(app: &mut App, underlying_token_address: Addr) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        lp_mock::execute,
        lp_mock::instantiate,
        lp_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(
        code,
        addr(api, ADMIN),
        &lp_mock::InstantiateMsg {
            underlying: underlying_token_address,
        },
        &[],
        "lp",
        None,
    )
    .unwrap()
}

mod incentives_mock {
    use astroport::incentives;
    use cosmwasm_std::to_json_binary;
    use cw_multi_test::IntoAddr;

    use super::*;
    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(_deps: Deps, _env: Env, msg: incentives::QueryMsg) -> StdResult<Binary> {
        match msg {
            incentives::QueryMsg::Config {} => to_json_binary(&incentives::Config {
                owner: "owner".into_addr(),
                factory: "factory".into_addr(),
                generator_controller: None,
                astro_token: AssetInfo::NativeToken {
                    denom: "astro".to_string(),
                },
                astro_per_second: Uint128::one(),
                total_alloc_points: Uint128::one(),
                vesting_contract: "vesting".into_addr(),
                guardian: None,
                incentivization_fee_info: None,
                token_transfer_gas_limit: None,
            }),
            _ => unimplemented!(),
        }
    }
}

fn instantiate_incentives(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        incentives_mock::execute,
        incentives_mock::instantiate,
        incentives_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(code, addr(api, ADMIN), &Empty {}, &[], "incentives", None)
        .unwrap()
}

fn proper_instantiate(
    app: &mut App,
    underlying_token_address: Addr,
    staking_address: Addr,
    lp_address: Addr,
    incentives_address: Addr,
) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        contract::execute,
        contract::instantiate,
        contract::query,
    )));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let oracle = addr(api, ORACLE);
    let msg = InstantiateMsg {
        manager: admin.clone(),
        oracle: oracle.clone(),
        share_name: "Share Token".to_string(),
        share_symbol: "sTKN".to_string(),
        share_marketing: None,
        underlying_token: AssetInfo::Token {
            contract_addr: underlying_token_address,
        },
        incentives: Vec::from([AssetInfo::NativeToken {
            denom: "incentive1".to_string(),
        }]),
        slippage_tolerance: Decimal::from_ratio(1_u32, 100_u32),
        staking_contract: Some(staking_address),
        lp: lp_address,
        tower_incentives: incentives_address,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw4626-escher", None)
        .unwrap()
}

#[test]
fn instantiates_properly() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let staking = instantiate_staking(&mut app);
    let lp = instantiate_lp(&mut app, asset.clone());
    let tower_incentives = instantiate_incentives(&mut app);
    proper_instantiate(&mut app, asset, staking, lp, tower_incentives);
}
