use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use crate::utils::asset::Contract;
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RewardTokenInfo{
    pub reward_token: Contract,
    pub daily_reward_amount: Uint128,
    pub valid_to: Uint128,
}
