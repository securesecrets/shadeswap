use fadroma::{
    scrt::{HumanAddr, StdResult, Api, CanonicalAddr},
    scrt_addr::{Canonize, Humanize},
    scrt_link::ContractLink,
};
use crate::{token_pair::TokenPair, custom_fee::Fee};
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

impl Canonize<AMMPair<CanonicalAddr>> for AMMPair<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<AMMPair<CanonicalAddr>> {
        Ok(AMMPair {
            pair: self.pair.canonize(api)?,
            address: self.address.canonize(api)?,
        })
    }
}

impl Humanize<AMMPair<HumanAddr>> for AMMPair<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<AMMPair<HumanAddr>> {
        Ok(AMMPair {
            pair: self.pair.humanize(api)?,
            address: api.human_address(&self.address)?,
        })
    }
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
            shade_dao_address: self.shade_dao_address.canonize(api)?
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

