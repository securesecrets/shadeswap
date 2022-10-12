use cosmwasm_std::{Binary, ContractInfo, QueryRequest, Response, StdError, StdResult, WasmQuery};
use secret_multi_test::App;
use serde::de::DeserializeOwned;

pub fn assert_error(response: StdResult<Response>, expected_msg: String) {
    match response {
        Ok(_) => panic!("Expected Error"),
        Err(err) => {
            assert_eq!(err, StdError::generic_err(expected_msg));
        }
    }
}

pub trait TestingExt {
    fn query_test<U: DeserializeOwned>(&self, contract: ContractInfo, msg: Binary) -> StdResult<U>;
}

impl TestingExt for App {
    fn query_test<U: DeserializeOwned>(&self, contract: ContractInfo, msg: Binary) -> StdResult<U> {
        self.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.to_string(),
            msg: msg,
            code_hash: contract.code_hash.clone(),
        }))
    }
}
