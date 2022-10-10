use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response, Uint128, Addr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::{core::{ContractInstantiationInfo, TokenType}};


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct StakingContractInit{
    pub contract_info: ContractInstantiationInfo,
    pub amount: Uint128,
    pub reward_token: TokenType    
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct ClaimableInfo{
    pub token_address: Addr,
    pub amount: Uint128
}
