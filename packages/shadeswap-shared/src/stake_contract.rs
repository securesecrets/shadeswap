use fadroma::{
    scrt::{Binary, Decimal, HumanAddr, Uint128},
    scrt_callback::Callback,
    scrt_link::{ContractInstantiationInfo, ContractLink},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::TokenType;

pub use crate::snip20_impl::msg as snip20;
use crate::token_amount::TokenAmount;
use crate::token_pair_amount::TokenPairAmount;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct StakingContractInit{
    pub contract_info: ContractInstantiationInfo,
    pub amount: Uint128,
    pub reward_token: TokenType<HumanAddr>,
    pub code_hash: String
}