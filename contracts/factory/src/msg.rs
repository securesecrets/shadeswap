use shadeswap_shared::amm_pair::AMMPair;
use shadeswap_shared::{Pagination, amm_pair::AMMSettings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::{fadroma::{
    scrt_callback::Callback,
    scrt_link::{ContractLink, ContractInstantiationInfo}, HumanAddr,
    scrt:: {
        Binary
    }
}, TokenPair};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings<HumanAddr>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetConfig {
        pair_contract: Option<ContractInstantiationInfo>,
    },
    CreateAMMPair { },
    AddAMMPairs {
        amm_pair: Vec<AMMPair<HumanAddr>>
    },
    RegisterAMMPair {
        pair: TokenPair<HumanAddr>,
        signature: Binary,
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    ListAMMPairs {
        pagination: Pagination
    },
    GetAMMPairAddress {
        pair: TokenPair<HumanAddr>
    },
    GetAMMSettings,
    GetConfig
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

