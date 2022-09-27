
use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
use cosmwasm_std::{
    to_binary, Addr, Empty, Binary, ContractInfo, StdResult, QueryRequest, WasmQuery,
};
use serde::de::DeserializeOwned;

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