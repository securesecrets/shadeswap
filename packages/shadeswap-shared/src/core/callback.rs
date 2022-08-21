

use cosmwasm_std::{Binary, CosmosMsg, WasmMsg, HumanAddr, CanonicalAddr};
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

impl Canonize for Callback<HumanAddr> 
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
    type Output = Callback<HumanAddr>;

    fn humanize(self, api: &impl cosmwasm_std::Api) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(Callback{
            msg: self.msg,
            contract: self.contract.humanize(api)?,
        })
    }
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
