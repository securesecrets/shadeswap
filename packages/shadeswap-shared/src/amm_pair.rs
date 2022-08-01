use fadroma::core::Humanize;
use fadroma::core::Canonize;
use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Extern,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage, Env, HandleResponse, 
    log,
    CanonicalAddr
};
use fadroma::core::ContractLink;

use crate::custom_fee::Fee;
use crate::token_pair::TokenPair;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct AMMPair<A: Clone> {
    /// The pair that the contract manages.
    pub pair: TokenPair<A>,
    /// Address of the contract that manages the exchange.
    pub address: A,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct AMMSettings<A> {
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub shade_dao_address: ContractLink<A>
}

impl AMMSettings<HumanAddr> {
    pub fn canonize(&self, api: &impl Api) -> StdResult<AMMSettings<CanonicalAddr>> {
        Ok(AMMSettings {
            lp_fee: self.lp_fee,
            shade_dao_fee: self.shade_dao_fee,
            shade_dao_address: self.shade_dao_address.clone().canonize(api)?
        })
    }
}

impl AMMSettings<CanonicalAddr> {
    pub fn humanize(self, api: &impl Api) -> StdResult<AMMSettings<HumanAddr>> {
        Ok(AMMSettings {
            lp_fee: self.lp_fee,
            shade_dao_fee: self.shade_dao_fee,
            shade_dao_address: self.shade_dao_address.humanize(api)?
        })
    }
}

