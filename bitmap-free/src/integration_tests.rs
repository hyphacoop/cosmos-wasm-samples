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
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }


    use crate::msg::{QueryMsg, GetGridResponse};

    fn proper_instantiate(x_size: u8, y_size: u8, z_values: String) -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let msg = if z_values.is_empty() {
            InstantiateMsg { x_size, y_size, z_values: None }
        } else {
            InstantiateMsg { x_size, y_size, z_values: Some(z_values) }
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
    fn test_instantiate_with_string() {
        let custom = "abcdef012345fedcba987654".to_string();
        let (app, cw_template_contract) = proper_instantiate(2, 2, custom.clone());
        let res: GetGridResponse = app.wrap().query_wasm_smart(
            cw_template_contract.addr(),
            &QueryMsg::GetGrid {},
        ).unwrap();
    assert_eq!(res.x_size, 2);
    assert_eq!(res.y_size, 2);
    assert_eq!(res.z_values, custom);
    }

    #[test]
    fn test_instantiate_with_none() {
        let x_size = 2;
        let y_size = 2;
        // Use proper_instantiate with an empty string to simulate None
        let (app, cw_template_contract) = proper_instantiate(x_size, y_size, String::new());
        let res: GetGridResponse = app.wrap().query_wasm_smart(
            cw_template_contract.addr(),
            &QueryMsg::GetGrid {},
        ).unwrap();
        assert_eq!(res.x_size, x_size);
        assert_eq!(res.y_size, y_size);
        assert_eq!(res.z_values, "0".repeat(x_size as usize * y_size as usize * 6));
    }

        #[test]
    fn test_set_point() {
        let x_size = 2;
        let y_size = 2;
        let (mut app, cw_template_contract) = proper_instantiate(x_size, y_size, String::new());
        // Set point (1,1) to "aabbcc"
        let z_value = "aabbcc".to_string();
        let msg = crate::msg::ExecuteMsg::Set { x: 1, y: 1, z: z_value.clone() };
        let cosmos_msg = cw_template_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        // Query point (1,1) and check value
        let point_res: crate::msg::GetPointResponse = app.wrap().query_wasm_smart(
            cw_template_contract.addr(),
            &crate::msg::QueryMsg::GetPoint { x: 1, y: 1 },
        ).unwrap();
        assert_eq!(point_res.point, z_value);

        // Query grid and check the value at the correct offset
        let grid_res: crate::msg::GetGridResponse = app.wrap().query_wasm_smart(
            cw_template_contract.addr(),
            &crate::msg::QueryMsg::GetGrid {},
        ).unwrap();
        let mut expected = "0".repeat(x_size as usize * y_size as usize * 6);
        let start = (1 * x_size as usize * 6) + (1 * 6);
        let end = start + 6;
        expected.replace_range(start..end, &z_value);
        assert_eq!(grid_res.z_values, expected);
    }

}
