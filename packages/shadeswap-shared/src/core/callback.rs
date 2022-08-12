

use cosmwasm_std::{Binary, CosmosMsg, WasmMsg, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ContractLink, Canonize};

#[derive(Serialize, Deserialize, Canonize, Clone, Debug, PartialEq, JsonSchema)]
/// Info needed to have the other contract respond.
pub struct Callback<A> {
    /// The message to call.
    pub msg: Binary,
    /// Info about the contract requesting the callback.
    pub contract: ContractLink<A>
}

impl Into<CosmosMsg> for Callback<HumanAddr> {
    fn into(self) -> CosmosMsg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract.address,
            callback_code_hash: self.contract.code_hash,
            msg: self.msg,
            send: vec![]
        })
    }
}
