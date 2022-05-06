use crate::msg::InitMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shadeswap_shared::{
    fadroma::{
        scrt_link::{ContractLink},    
        scrt_addr::{Humanize, Canonize},
        scrt::{
            Api, CanonicalAddr, Extern, HumanAddr, Uint128,
            Querier, StdResult, Storage, StdError
        },
        scrt_storage::{load, save},
        scrt_vk::ViewingKey,
    },
    amm_pair::AMMPair, TokenPair
};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A: Clone> {
    pub factory_address: ContractLink<A>
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            factory_address: self.factory_address.canonize(api)?
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            factory_address: self.factory_address.humanize(api)?
        })
    }
}

pub fn config_write<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>,
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub fn config_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Config<HumanAddr>> {
    let config: Option<Config<CanonicalAddr>> = load(&deps.storage, CONFIG_KEY)?;
    config
        .ok_or(StdError::generic_err("Config doesn't exist in storage."))?
        .humanize(&deps.api)
}

impl Config<HumanAddr> {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            factory_address: msg.factory_address
        }
    }
}