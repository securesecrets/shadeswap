use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response, 
    CanonicalAddr
};
use crate::{custom_fee::Fee, core::Humanize};
use crate::token_pair::TokenPair;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::core::{Canonize, ContractLink};

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct AMMPair<A: Clone> {
    /// The pair that the contract manages.
    pub pair: TokenPair<A>,
    /// Address of the contract that manages the exchange.
    pub address: A,
}

impl Canonize for AMMPair<String> {
    fn canonize(self, api: &impl Api) -> StdResult<AMMPair<CanonicalAddr>> {
        Ok(AMMPair {
            pair: self.pair.canonize(api)?,
            address: self.address.canonize(api)?,
        })
    }

    type Output = AMMPair<CanonicalAddr>;
}

impl Humanize for AMMPair<CanonicalAddr> {
    fn humanize(self, api: &impl Api) -> StdResult<AMMPair<String>> {
        Ok(AMMPair {
            pair: self.pair.humanize(api)?,
            address: self.address.humanize(api)?,
        })
    }

    type Output = AMMPair<String>;
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug,Clone)]
pub struct AMMSettings<A> {
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub shade_dao_address: ContractLink<A>
}

impl Canonize for AMMSettings<String> {
    fn canonize(self, api: &impl Api) -> StdResult<AMMSettings<CanonicalAddr>> {
        Ok(AMMSettings {
            lp_fee: self.lp_fee,
            shade_dao_fee: self.shade_dao_fee,
            shade_dao_address: self.shade_dao_address.canonize(api)?
        })
    }

    type Output = AMMSettings<CanonicalAddr>;
}

impl Humanize for AMMSettings<CanonicalAddr> {
    fn humanize(self, api: &impl Api) -> StdResult<AMMSettings<String>> {
        Ok(AMMSettings {
            lp_fee: self.lp_fee,
            shade_dao_fee: self.shade_dao_fee,
            shade_dao_address: self.shade_dao_address.humanize(api)?
        })
    }

    type Output = AMMSettings<String>;
}

