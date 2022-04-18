use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use amm_
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
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

pub mod amm_pair {
    use crate::amm_pair::AMMPair;
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    use secret_toolkit::utils::{InitCallback, HandleCallback, Query};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct InitMsg {       
        pub amm_pair: AMMPair<HumanAddr>,
        pub lp_token_contract: ContractInstantiationInfo,      
        pub factory_info: ContractLink<HumanAddr>,
        pub callback: Callback<HumanAddr>,
        pub prng_seed: Binary,
        pub entropy: Binary,
    }
    
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        AddLiquidityToAMMContract {
            deposit: TokenPairAmount<HumanAddr>,
        },
        SwapTokens {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
            expected_return: Option<Uint128>,
            to: Option<HumanAddr>,
        },
        // SNIP20 receiver interface
        ReceiveCallback {
            from: HumanAddr,
            msg: Option<Binary>,
            amount: Uint128,
        },
        // Sent by the LP token contract so that we can record its address.
        OnLpTokenInitAddr
    }
    
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        SwapTokens {
            expected_return: Option<Uint128>,
            to: Option<HumanAddr>,
        },
        RemoveLiquidity {
            recipient: HumanAddr,
        },
    }
    
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        PairInfo,    
    }
    
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        PairInfo {
            liquidity_token: ContractLink<HumanAddr>,
            factory: ContractLink<HumanAddr>,
            pair: TokenPair<HumanAddr>,
            amount_0: Uint128,
            amount_1: Uint128,
            total_liquidity: Uint128,
            contract_version: u32,
        },
    }
   
}

pub mod factory {
    use crate::{fadroma::HumanAddr, amm_pair::AMMSettings};
    use fadroma::ContractInstantiationInfo;
    use schemars::JsonSchema;
    use serde::{Serialize, Deserialize};

    use crate::amm_pair::AMMPair;

    
    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse{
        ListAMMPairs {
            amm_pairs: Vec<AMMPair<HumanAddr>>,
        },
        Config {
            pair_contract: ContractInstantiationInfo,
            amm_settings: AMMSettings<HumanAddr>
        } 
    }
}