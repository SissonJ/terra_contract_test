#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Variable};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra_contract_test";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {};
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Hydrate { msg, vars } => try_hydrate(msg, vars),
    }
}

pub fn try_hydrate(msg: String, vars: String) -> Result<Response, ContractError> {
    // Set up vec to add messages to
    let mut msgs_to_send: Vec<CosmosMsg> = vec![];
    // Deserialize vars
    let modified_vars = vars.replace(":", ",").replace(" ", "").replace(",]", "]");
    println!("This one {}", modified_vars);
    let vars_vec: Vec<String> = match serde_json_wasm::from_str(&modified_vars) {
        Ok(val) => val,
        Err(error) => {
            println!("Ther error is: {:?}", error);
            vec![]
        }
    };
    // start unbinarying the msg
    let mut msg_substring = msg.replace(" ", "").clone();
    let mut decoded_msg = "".to_string();
    let mut postscript = "".to_string();
    while msg_substring.find("msg").is_some() {
        println!("msg_substring: {}", msg_substring);
        match msg_substring.find("msg") {
            Some(first_index) => {
                let second_index = match msg_substring[first_index + 5..msg_substring.len()]
                    .to_string()
                    .find(",")
                {
                    Some(second_index) => second_index + first_index + 5,
                    None => {
                        return Err(ContractError::CustomError {
                            val: "unable to deserialize".to_string(),
                        })
                    }
                };
                let prescript = msg_substring[0..first_index + 5].to_string();
                if postscript.eq(&"".to_string()) {
                    postscript = msg_substring[second_index..msg_substring.len()].to_string();
                }
                let base64_string = serde_json_wasm::from_str::<String>(
                    &msg_substring[first_index + 5..second_index],
                )?;
                println!("base64_string: {}", base64_string);
                msg_substring = serde_json_wasm::to_string(&Binary::from_base64(&base64_string)?)?;
                decoded_msg = format!("{}{}{}", decoded_msg, prescript.to_string(), msg_substring,);
            }
            None => (),
        }
    }
    decoded_msg = format!("{}{}", decoded_msg, postscript);
    println!("vars: {:?}, msg: {:?}", vars_vec, decoded_msg);
    for (i, vars) in vars_vec.iter().enumerate() {
        if i % 2 == 0 {
            decoded_msg = decoded_msg.replace(vars, &vars_vec[i + 1]);
        }
    }
    encoded_msg = {
        decoded_msg.rfind("msg").iter().next()
    }
    Ok(Response::new()
        .add_messages(msgs_to_send)
        .add_attribute("method", "try_increment"))
}

/*
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    to_binary(&String::from("Query Successful"))
}

/*fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_hydration() {
        let test_vars = String::from(
            "[\"$warp.var.variable1\": \"terra12345\",\"$warp.var.variable2\": \"uterra\",\"$warp.var.variable3\": \"54321\",\"$warp.var.variable4\": \"terra11111\",\"$warp.var.variable5\": \"0.05\"]",
        );
        let test_msg = "{\"wasm\": {\"execute\": {\"contract_addr\": \"$warp.var.variable1\",\"msg\":\"eyJzZW5kIjp7ImNvbnRyYWN0IjoidGVycmE1NDMyMSIsImFtb3VudCI6IjEyMzQ1IiwibXNnIjoiZXlKbGVHVmpkWFJsWDNOM1lYQmZiM0JsY21GMGFXOXVjeUk2ZXlKdmNHVnlZWFJwYjI1eklqcGJleUpoYzNSeWIxOXpkMkZ3SWpwN0ltOW1abVZ5WDJGemMyVjBYMmx1Wm04aU9uc2lkRzlyWlc0aU9uc2lZMjl1ZEhKaFkzUmZZV1JrY2lJNklpUjNZWEp3TG5aaGNpNTJZWEpwWVdKc1pURWlmWDBzSW1GemExOWhjM05sZEY5cGJtWnZJanA3SW01aGRHbDJaVjkwYjJ0bGJpSTZleUprWlc1dmJTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVElpZlgxOWZWMHNJbTFwYm1sdGRXMWZjbVZqWldsMlpTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVE1pTENKMGJ5STZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVFFpTENKdFlYaGZjM0J5WldGa0lqb2lKSGRoY25BdWRtRnlMblpoY21saFlteGxOU0o5ZlE9PSJ9fQ==\",\"funds\": []}}}".to_string();
        let modified_test_vars = test_vars.replace(":", ",");
        println!("{:?}", modified_test_vars);
        try_hydrate(test_msg, test_vars.clone()).unwrap();
        let vars: Vec<String> = match serde_json_wasm::from_str(&modified_test_vars) {
            Ok(vars) => vars,
            Err(err) => {
                print!("{:?}", err);
                assert!(false, "serde error");
                vec![]
            }
        };
        print!("{:?}", vars);
        assert!(false)
    }
}

/*#[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}*/
