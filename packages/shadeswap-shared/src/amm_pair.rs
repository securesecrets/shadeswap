use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response, 
    CanonicalAddr, Addr
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::core::{ContractLink, TokenPair, Fee};

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct AMMPair {
    /// The pair that the contract manages.
    pub pair: TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: Addr,
}


#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug,Clone)]
pub struct AMMSettings {
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub shade_dao_address: ContractLink
}

