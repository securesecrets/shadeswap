use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::{
    amm_pair:: {
        AMMPair, AMMSettings
    },
    fadroma::{
        scrt::{
            Api, Binary, CanonicalAddr, Extern, HumanAddr,
            Querier, StdError, StdResult, Storage
        },
        scrt_addr::{Canonize, Humanize},
        scrt_link::{ContractInstantiationInfo, ContractLink},
        scrt_storage::{load, ns_load, ns_remove, ns_save, save},
    },
    Pagination,
};

use secret_toolkit::utils::{HandleCallback, InitCallback, Query};

use crate::msg::InitMsg;

const NS_EXCHANGES: &[u8] = b"exchanges";

pub static CONFIG_KEY: &[u8] = b"config";
pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A> {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings<A>
}

impl Config<HumanAddr> {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            pair_contract: msg.pair_contract,
            amm_settings: msg.amm_settings,
        }
    }
}
impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            pair_contract: self.pair_contract.clone(),
            amm_settings: self.amm_settings.canonize(api)?,
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            pair_contract: self.pair_contract.clone(),
            amm_settings: self.amm_settings.clone().humanize(api)?,
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

pub(crate) fn get_amm_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Vec<AMMPair<HumanAddr>>> {
    let count = 0;

    if pagination.start >= count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let exchange: AMMPair<CanonicalAddr> =
            ns_load(&deps.storage, NS_EXCHANGES, i.to_string().as_bytes())?
                .ok_or_else(|| StdError::generic_err("AMMPair doesn't exist in storage."))?;

        result.push(exchange.humanize(&deps.api)?);
    }

    Ok(result)
}
