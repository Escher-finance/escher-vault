#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::Addr;
    use cosmwasm_std::Uint128;
    use cw4626::cw20::TokenInfoResponse;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::contract;
    use crate::msg::*;
    use cw4626::*;

    const USER: &str = "user";
    const ADMIN: &str = "admin";

    fn addr(api: &MockApi, addr: &str) -> Addr {
        api.addr_make(addr)
    }

    fn get_app() -> App {
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
            mint: None,
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
    fn instantiates_properly() {
        let mut app = get_app();
        let asset = instantitate_asset(&mut app);
        let vault = proper_instantiate(&mut app, asset.clone());
        let api = app.api();
        let querier = app.wrap();
        let user = addr(api, USER);
        assert_eq!(
            querier
                .query_wasm_smart::<AssetResponse>(&vault, &QueryMsg::Asset {})
                .unwrap()
                .asset_token_address,
            asset,
            "underlying asset address must match"
        );
        let share_token_info = querier
            .query_wasm_smart::<TokenInfoResponse>(&vault, &QueryMsg::TokenInfo {})
            .unwrap();
        let asset_token_info = querier
            .query_wasm_smart::<TokenInfoResponse>(&asset, &QueryMsg::TokenInfo {})
            .unwrap();
        assert_eq!(
            share_token_info.decimals, asset_token_info.decimals,
            "asset and share must have the same decimals"
        );
        assert_eq!(
            share_token_info.total_supply,
            Uint128::zero(),
            "initial total share supply must be zero",
        );
        assert_eq!(
            querier
                .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
                .unwrap()
                .total_managed_assets,
            Uint128::zero(),
            "initial total managed assets must be zero"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
                .unwrap()
                .owner
                .unwrap(),
            addr(api, ADMIN),
            "admin must be set"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<ConvertToSharesResponse>(
                    &vault,
                    &QueryMsg::ConvertToShares {
                        assets: 1000u128.into()
                    }
                )
                .unwrap()
                .shares,
            Uint128::zero(),
            "initial asset to share conversion must yield zero"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<ConvertToAssetsResponse>(
                    &vault,
                    &QueryMsg::ConvertToAssets {
                        shares: 1000u128.into()
                    }
                )
                .unwrap()
                .assets,
            Uint128::zero(),
            "initial share to asset conversion must yield zero"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<MaxDepositResponse>(
                    &vault,
                    &QueryMsg::MaxDeposit {
                        receiver: user.clone()
                    }
                )
                .unwrap()
                .max_assets,
            Uint128::MAX,
            "max deposit must not be limited"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<MaxMintResponse>(
                    &vault,
                    &QueryMsg::MaxMint {
                        receiver: user.clone()
                    }
                )
                .unwrap()
                .max_shares,
            Uint128::MAX,
            "max mint must not be limited"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<MaxWithdrawResponse>(
                    &vault,
                    &QueryMsg::MaxWithdraw {
                        owner: user.clone()
                    }
                )
                .unwrap()
                .max_assets,
            Uint128::zero(),
            "initial max withdraw must be zero"
        );
        assert_eq!(
            querier
                .query_wasm_smart::<MaxRedeemResponse>(
                    &vault,
                    &QueryMsg::MaxRedeem {
                        owner: user.clone()
                    }
                )
                .unwrap()
                .max_shares,
            Uint128::zero(),
            "initial max redeem must be zero"
        );
    }
}
