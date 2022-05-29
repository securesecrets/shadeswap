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
        scrt_vk::ViewingKey, Env, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlyStorage, Binary,
    },
    amm_pair::AMMPair, TokenPair, TokenAmount, TokenType,
    msg::router::InitMsg
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ADDED_TOKEN_LIST: &[u8] = b"added_token_list";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A: Clone> {
    pub factory_address: ContractLink<A>,
    pub viewing_key: ViewingKey
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            factory_address: self.factory_address.canonize(api)?,
            viewing_key: self.viewing_key.clone()
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            factory_address: self.factory_address.humanize(api)?,
            viewing_key: self.viewing_key.clone()
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

pub fn write_new_token<S: Storage>(store: &mut S, token_address: &CanonicalAddr, key: &ViewingKey) {
    let mut balance_store = PrefixedStorage::new(ADDED_TOKEN_LIST, store);
    balance_store.set(token_address.as_slice(), &key.to_hashed());
}

pub fn read_token<S: Storage>(store: &S, token_address: &CanonicalAddr) -> Option<Vec<u8>> {
    let balance_store = ReadonlyPrefixedStorage::new(ADDED_TOKEN_LIST, store);
    balance_store.get(token_address.as_slice())
}

impl Config<HumanAddr> {
    pub fn from_init_msg(env:Env,msg: InitMsg) -> Self {
        let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());
        Self {
            factory_address: msg.factory_address,
            viewing_key: viewing_key
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount<HumanAddr>,
    pub paths: Vec<HumanAddr>,
    pub signature: Binary,
    pub recipient: HumanAddr,
    pub current_index: u32
}