use cosmwasm_std::{
    Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::core::Humanize;
use shadeswap_shared::scrt_storage::{load, ns_load, ns_save, save};
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings},
    core::{Canonize, ContractInstantiationInfo},
    msg::factory::InitMsg,
    Pagination, TokenPair, TokenType,
};

use secret_toolkit::utils::{HandleCallback, InitCallback, Query};

const NS_AMM_PAIRS: &[u8] = b"amm_pairs";
const AMM_PAIR_COUNT_KEY: &[u8] = b"amm_pairs_count";
const PRNG_KEY: &[u8] = b"prng_seed";

pub static CONFIG_KEY: &[u8] = b"config";
pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A> {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings<A>,
    pub lp_token_contract: ContractInstantiationInfo,
}

impl Config<HumanAddr> {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            pair_contract: msg.pair_contract,
            amm_settings: msg.amm_settings,
            lp_token_contract: msg.lp_token_contract,
        }
    }
}
impl Canonize for Config<HumanAddr> {
    fn canonize(self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            pair_contract: self.pair_contract.clone(),
            amm_settings: self.amm_settings.canonize(api)?,
            lp_token_contract: self.lp_token_contract.clone(),
        })
    }
    type Output = Config<CanonicalAddr>;
}

impl Humanize for Config<CanonicalAddr> {
    fn humanize(self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            pair_contract: self.pair_contract.clone(),
            amm_settings: self.amm_settings.clone().humanize(api)?,
            lp_token_contract: self.lp_token_contract.clone(),
        })
    }
    type Output = Config<HumanAddr>;
}

pub fn config_write<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: Config<HumanAddr>,
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

pub(crate) fn generate_pair_key(pair: &TokenPair<CanonicalAddr>) -> Vec<u8> {
    let mut bytes: Vec<&[u8]> = Vec::new();

    match &pair.0 {
        TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice()),
    }

    match &pair.1 {
        TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice()),
    }

    bytes.sort();

    bytes.concat()
}

pub(crate) fn save_amm_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    exchanges: Vec<AMMPair<HumanAddr>>,
) -> StdResult<()> {
    let mut count = load_amm_pairs_count(&deps.storage)?;

    for exchange in exchanges {
        let exchange = exchange.canonize(&deps.api)?;
        let key = generate_pair_key(&exchange.pair);

        let result: Option<CanonicalAddr> = ns_load(&deps.storage, NS_AMM_PAIRS, &key)?;
        if result.is_some() {
            return Err(StdError::generic_err(format!(
                "Exchange ({}) already exists",
                exchange.pair
            )));
        }
        ns_save(&mut deps.storage, NS_AMM_PAIRS, &key, &exchange.address)?;
        ns_save(
            &mut deps.storage,
            NS_AMM_PAIRS,
            count.to_string().as_bytes(),
            &exchange,
        )?;
        count += 1;
    }

    save_amm_pairs_count(&mut deps.storage, count)
}

pub(crate) fn get_address_for_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TokenPair<HumanAddr>,
) -> StdResult<HumanAddr> {
    let key = generate_pair_key(&pair.canonize(&deps.api)?);

    let canonical = ns_load(&deps.storage, NS_AMM_PAIRS, &key)?
        .ok_or_else(|| StdError::generic_err("Address doesn't exist in storage."))?;

    deps.api.human_address(&canonical)
}

pub(crate) fn load_amm_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Vec<AMMPair<HumanAddr>>> {
    let count = load_amm_pairs_count(&deps.storage)?;

    if pagination.start >= count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let exchange: AMMPair<CanonicalAddr> =
            ns_load(&deps.storage, NS_AMM_PAIRS, i.to_string().as_bytes())?
                .ok_or_else(|| StdError::generic_err("AMMPair doesn't exist in storage."))?;

        result.push(exchange.humanize(&deps.api)?);
    }

    Ok(result)
}

#[inline]
pub fn load_amm_pairs_count(storage: &impl Storage) -> StdResult<u64> {
    Ok(load(storage, AMM_PAIR_COUNT_KEY)?.unwrap_or(0))
}

#[inline]
pub fn save_amm_pairs_count(storage: &mut impl Storage, count: u64) -> StdResult<()> {
    save(storage, AMM_PAIR_COUNT_KEY, &count)
}

pub(crate) fn load_prng_seed(storage: &impl Storage) -> StdResult<Binary> {
    let prng_seed: Option<Binary> = load(storage, PRNG_KEY)?;
    prng_seed.ok_or(StdError::generic_err("Prng seed doesn't exist in storage."))
}

pub(crate) fn save_prng_seed(storage: &mut impl Storage, prng_seed: &Binary) -> StdResult<()> {
    save(storage, PRNG_KEY, prng_seed)
}
