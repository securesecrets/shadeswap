use cosmwasm_std::Api;
use cosmwasm_std::Binary;
use cosmwasm_std::CanonicalAddr;
use cosmwasm_std::Env;
use cosmwasm_std::Extern;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::Querier;
use cosmwasm_std::Storage;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::Uint128;
use cosmwasm_storage::PrefixedStorage;
use cosmwasm_storage::ReadonlyPrefixedStorage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::core::Humanize;
use shadeswap_shared::scrt_storage::load;
use shadeswap_shared::scrt_storage::save;
use shadeswap_shared::{core::Canonize, viewing_keys::ViewingKey};

use shadeswap_shared::{
    amm_pair::AMMPair, core::ContractLink, msg::router::InitMsg, TokenAmount,
    TokenPair, TokenType,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ADDED_TOKEN_LIST: &[u8] = b"added_token_list";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A> {
    pub factory_address: ContractLink<A>,
    pub viewing_key: String,
}

impl Config<HumanAddr> {
    pub fn from_init_msg(env: Env, msg: InitMsg) -> Self {
        let viewing_key =
            ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice()).to_string();
        Self {
            factory_address: msg.factory_address,
            viewing_key: viewing_key,
        }
    }
}

impl Canonize for Config<HumanAddr> {
    fn canonize(self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            factory_address: self.factory_address.canonize(api)?,
            viewing_key: self.viewing_key.clone(),
        })
    }

    type Output = Config<CanonicalAddr>;
}

impl Humanize for Config<CanonicalAddr> {
    fn humanize(self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            factory_address: self.factory_address.humanize(api)?,
            viewing_key: self.viewing_key.clone(),
        })
    }

    type Output = Config<HumanAddr>;
}

pub fn config_write<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    config: Config<HumanAddr>,
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub fn config_read<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount<HumanAddr>,
    pub amount_out_min: Option<Uint128>,
    pub paths: Vec<HumanAddr>,
    pub signature: Binary,
    pub recipient: HumanAddr,
    pub current_index: u32,
}
