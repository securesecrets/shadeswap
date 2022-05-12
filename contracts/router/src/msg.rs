use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::{fadroma::{ContractLink, HumanAddr, Uint128, Binary}, TokenAmount, TokenType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub factory_address: ContractLink<HumanAddr>,
    pub prng_seed: Binary,
    pub entropy: Binary
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SwapTokens {
        /// The token type to swap from.
        offer: TokenAmount<HumanAddr>,
        expected_return: Option<Uint128>,
        path: Vec<TokenType<HumanAddr>>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
