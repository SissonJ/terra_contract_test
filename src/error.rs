use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}

impl From<serde_json_wasm::de::Error> for ContractError {
    fn from(_: serde_json_wasm::de::Error) -> Self {
        ContractError::CustomError {
            val: String::from("DeserializationError"),
        }
    }
}

impl From<serde_json_wasm::ser::Error> for ContractError {
    fn from(_: serde_json_wasm::ser::Error) -> Self {
        ContractError::CustomError {
            val: String::from("DeserializationError"),
        }
    }
}
