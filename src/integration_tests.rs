#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
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
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {};
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

    mod hydrate {
        use super::*;
        use crate::msg::ExecuteMsg;

        #[test]
        fn hydrate() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let test_vars = "[\"$warp.var.variable1\": \"terra12345\",\"$warp.var.variable2\": \"uterra\",\"$warp.var.variable3\": \"54321\",\"$warp.var.variable4\": \"terra11111\",\"$warp.var.variable5\": \"0.05\",]".to_string();
            let test_msg = "{\"wasm\": {\"execute\": {\"contract_addr\": \"$warp.var.variable1\",\"msg\":\"eyJzZW5kIjp7ImNvbnRyYWN0IjoidGVycmE1NDMyMSIsImFtb3VudCI6IjEyMzQ1IiwibXNnIjoiZXlKbGVHVmpkWFJsWDNOM1lYQmZiM0JsY21GMGFXOXVjeUk2ZXlKdmNHVnlZWFJwYjI1eklqcGJleUpoYzNSeWIxOXpkMkZ3SWpwN0ltOW1abVZ5WDJGemMyVjBYMmx1Wm04aU9uc2lkRzlyWlc0aU9uc2lZMjl1ZEhKaFkzUmZZV1JrY2lJNklpUjNZWEp3TG5aaGNpNTJZWEpwWVdKc1pURWlmWDBzSW1GemExOWhjM05sZEY5cGJtWnZJanA3SW01aGRHbDJaVjkwYjJ0bGJpSTZleUprWlc1dmJTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVElpZlgxOWZWMHNJbTFwYm1sdGRXMWZjbVZqWldsMlpTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVE1pTENKMGJ5STZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVFFpTENKdFlYaGZjM0J5WldGa0lqb2lKSGRoY25BdWRtRnlMblpoY21saFlteGxOU0o5ZlE9PSJ9fQ==\",\"funds\": []}}}".to_string();

            let msg = ExecuteMsg::Hydrate {
                msg: test_msg,
                vars: test_vars,
            };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        }
    }
}
