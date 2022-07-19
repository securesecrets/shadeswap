use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Extern,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage, Env, HandleResponse, log, Uint128,
};
use fadroma::prelude::ContractInstantiationInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::TokenType;


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct StakingContractInit{
    pub contract_info: ContractInstantiationInfo,
    pub amount: Uint128,
    pub reward_token: TokenType<HumanAddr>    
}