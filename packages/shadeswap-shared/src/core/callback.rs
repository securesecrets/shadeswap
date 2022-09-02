

use cosmwasm_std::{Binary, CosmosMsg, WasmMsg, CanonicalAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ContractLink, Canonize, Humanize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Info needed to have the other contract respond.
pub struct Callback<A> {
    /// The message to call.
    pub msg: Binary,
    /// Info about the contract requesting the callback.
    pub contract: ContractLink<A>
}

impl Canonize for Callback<String> 
{
    type Output = Callback<CanonicalAddr>;

    fn canonize(self, api: &impl cosmwasm_std::Api) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(Callback{
            msg: self.msg,
            contract: self.contract.canonize(api)?,
        })
    }
}

impl Humanize for Callback<CanonicalAddr> 
{
    type Output = Callback<String>;

    fn humanize(self, api: &impl cosmwasm_std::Api) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(Callback{
            msg: self.msg,
            contract: self.contract.humanize(api)?,
        })
    }
}

impl Into<CosmosMsg> for Callback<String> {
    fn into(self) -> CosmosMsg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract.address,
            code_hash: self.contract.code_hash,
            msg: self.msg,
            funds: None,
        })
    }
}
