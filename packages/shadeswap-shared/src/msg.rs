use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    use secret_toolkit::utils::{InitCallback, HandleCallback, Query};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
    }

    impl InitCallback for InitMsg {
        const BLOCK_SIZE: usize = 256;
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
        },
        GetAMMPairAddress {
            address: HumanAddr
        },
        GetAMMSettings {
            settings: AMMSettings<HumanAddr>,
        }
    }
}