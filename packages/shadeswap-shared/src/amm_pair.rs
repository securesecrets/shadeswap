use fadroma::{
    scrt::{HumanAddr, StdResult, Api, CanonicalAddr},
    scrt_addr::{Canonize, Humanize}
};
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
    pub swap_fee: Fee,
    pub shadeswap_fee: Fee,
    pub shadeswap_burner: Option<A>,
}

impl AMMSettings<HumanAddr> {
    pub fn canonize(&self, api: &impl Api) -> StdResult<AMMSettings<CanonicalAddr>> {
        Ok(AMMSettings {
            swap_fee: self.swap_fee,
            shadeswap_fee: self.shadeswap_fee,
            shadeswap_burner: if let Some(info) = &self.shadeswap_burner {
                Some(info.canonize(api)?)
            } else {
                None
            },
        })
    }
}

impl AMMSettings<CanonicalAddr> {
    pub fn humanize(self, api: &impl Api) -> StdResult<AMMSettings<HumanAddr>> {
        Ok(AMMSettings {
            swap_fee: self.swap_fee,
            shadeswap_fee: self.shadeswap_fee,
            shadeswap_burner: if let Some(info) = self.shadeswap_burner {
                Some(info.humanize(api)?)
            } else {
                None
            },
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16,
}

impl Fee {
    pub fn new(nom: u8, denom: u16) -> Self {
        Self { nom, denom }
    }
}
