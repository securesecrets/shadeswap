use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};

use shadeswap_shared::fadroma::scrt_link::{ContractInstantiationInfo, ContractLink};

use crate::msg::InitMsg;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub pair_contract: ContractInstantiationInfo,
}

pub fn config_write<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

impl State {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            pair_contract: msg.pair_contract,
        }
    }
}