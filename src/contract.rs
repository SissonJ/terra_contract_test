#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg,
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
    let vars_vec: Vec<String> = match serde_json_wasm::from_str(&modified_vars) {
        Ok(val) => val,
        Err(error) => {
            vec![]
        }
    };
    // start decoding the msg
    let mut msg_substring = msg.replace(" ", "").clone();
    let mut decoded_msg = "".to_string();
    let mut postscript = "".to_string();
    // we want to find the message to isolate the b64 encoded stuff
    // the loop will end when we've decoded all the b64 msgs
    while msg_substring.find("msg").is_some() {
        match msg_substring.find("msg") {
            Some(first_index) => {
                // The second index will be the , because inbetween the comma and "msg" is the b64
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
                // save what come before "msg"
                let prescript = msg_substring[0..first_index + 5].to_string();
                // save the last part, we only need to do this once though
                if postscript.eq(&"".to_string()) {
                    postscript = msg_substring[second_index..msg_substring.len()].to_string();
                }
                // decrypt the data now that we have it isolated
                let base64_string = serde_json_wasm::from_str::<String>(
                    &msg_substring[first_index + 5..second_index],
                )?;
                msg_substring = serde_json_wasm::to_string(&Binary::from_base64(&base64_string)?)?;
                // put it all back together
                decoded_msg = format!("{}{}{}", decoded_msg, prescript.to_string(), msg_substring,);
                // If there's a nested msg, the find("msg") will be able to find something and
                // we'll do it over again, satisfying the nested msg req
            }
            None => (),
        }
    }
    // Once it's all decoded, we want to replace the variables with the data
    decoded_msg = format!("{}{}", decoded_msg, postscript);
    for (i, vars) in vars_vec.iter().enumerate() {
        // Do this operation every other loop since the array is organized [name, val, name, val..]
        if i % 2 == 0 {
            decoded_msg = decoded_msg.clone().replace(vars, &vars_vec[i + 1]);
        }
    }
    // rencode the msg
    let encoded_msg = {
        let mut encoded_msg = decoded_msg.clone();
        // find the last instance of "msg" as this will be the deepest nested msg
        let mut msg_iter = decoded_msg.rfind("msg").into_iter();
        let mut next_msg_index = msg_iter.next();
        let mut index = 0;
        while next_msg_index.clone() != None {
            // we want this to execute every other since first operation encoded the msg, then
            // since we're not encoding "msg" we need to skip one loop to move on
            if index % 2 == 0 {
                // isolate the msg part
                let second_index =
                    decoded_msg[next_msg_index.unwrap().clone()..decoded_msg.len()].find(",");
                let encoded_nested_msg = to_binary(
                    &encoded_msg[next_msg_index.unwrap().clone() + 5..second_index.unwrap()]
                        .to_string(),
                )?;
                // encode it and replace the old msg wit the encoded msg
                encoded_msg = encoded_msg.replace(
                    &encoded_msg[next_msg_index.unwrap().clone() + 5..second_index.unwrap()]
                        .to_string(),
                    &encoded_nested_msg.to_string(),
                )
            }
            next_msg_index = msg_iter.next();
            index = index + 1;
        }
        encoded_msg
    };
    // deserialize to a cosmos msg and add it to the response
    let cosmos_msg: CosmosMsg = serde_json_wasm::from_str(&encoded_msg)?;
    Ok(Response::new()
        .add_message(cosmos_msg)
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

/*#[cfg(test)]
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

*/

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
