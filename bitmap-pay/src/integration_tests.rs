#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1_000_000),
                    }],
                )
                .unwrap();
        })
    }

    use crate::msg::{QueryMsg, GetPointResponse, GetGridResponse};

    fn proper_instantiate(x_size: u8, y_size: u8) -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let grid_len = (x_size as usize) * (y_size as usize) * 6;
        let nonzero_z = "aabbcc".repeat((grid_len + 5) / 6).chars().take(grid_len).collect::<String>();
        let msg = InstantiateMsg {
            x_size,
            y_size,
            z_values: Some(nonzero_z.clone()),
            recipient: ADMIN.to_string(),
            supply_base_fee: 100,
            supply_fee_factor: 10,
            update_base_fee: 100,
            update_fee_factor: 10,
            fee_denom: NATIVE_DENOM.to_string(),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();
        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);
        (app, cw_template_contract)
    }

    #[test]
    fn test_instantiate() {
        let (app, cw_template_contract) = proper_instantiate(2, 2);
        // Query grid and check all zeroes
        let res: GetGridResponse = app.wrap().query_wasm_smart(
            cw_template_contract.addr(),
            &QueryMsg::GetGrid {},
        ).unwrap();
    assert_eq!(res.x_size, 2);
    assert_eq!(res.y_size, 2);
    let expected_z = "aabbcc".repeat(2*2).chars().take(2*2*6).collect::<String>();
    assert_eq!(res.z_values, expected_z);

    let res: GetPointResponse = app.wrap().query_wasm_smart(
        cw_template_contract.addr(),
        &QueryMsg::GetPoint { x: 1, y: 1 },
    ).unwrap();
    assert_eq!(res.point, "aabbcc".to_string());
    }

}
