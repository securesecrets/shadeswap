

use cosmwasm_std::{Binary, CosmosMsg, WasmMsg, CanonicalAddr, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ContractLink};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Info needed to have the other contract respond.
pub struct Callback {
    /// The message to call.
    pub msg: Binary,
    /// Info about the contract requesting the callback.
    pub contract: ContractLink
}

impl Into<CosmosMsg> for Callback {
    fn into(self) -> CosmosMsg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract.address.into_string(),
            code_hash: self.contract.code_hash,
            msg: self.msg,
            funds: vec![]
        })
    }
}
