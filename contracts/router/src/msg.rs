use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::{
    fadroma::{Binary, ContractLink, HumanAddr, Uint128},
    TokenAmount, TokenType,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub factory_address: ContractLink<HumanAddr>,
    pub prng_seed: Binary,
    pub entropy: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128,
    },
    SwapTokensForExact {
        /// The token type to swap from.
        offer: TokenAmount<HumanAddr>,
        expected_return: Option<Uint128>,
        path: Vec<HumanAddr>,
    },
    SwapCallBack {
        current_index: usize,
        last_token_in: TokenType<HumanAddr>,
        signature: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}