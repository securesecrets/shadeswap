use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Storage;
use cosmwasm_std::Uint128;
use cosmwasm_storage::ReadonlySingleton;
use cosmwasm_storage::Singleton;
use cosmwasm_storage::singleton;
use cosmwasm_storage::singleton_read;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shadeswap_shared::core::TokenAmount;
use shadeswap_shared::{
    amm_pair::AMMPair, core::ContractLink, msg::router::InitMsg
};

pub static CONFIG: &[u8] = b"config";
pub static ADDED_TOKEN_LIST: &[u8] = b"added_token_list";
pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub factory_address: ContractLink,
    pub viewing_key: String,
}



pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}


pub fn added_tokens_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn added_tokens_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

pub fn epheral_storage_w(storage: &mut dyn Storage) -> Singleton<CurrentSwapInfo> {
    singleton(storage, CONFIG)
}

pub fn epheral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<CurrentSwapInfo> {
    singleton_read(storage, CONFIG)
}
/*
pub fn config_write<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    config: Config,
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub fn config_read<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>,
) -> StdResult<Config> {
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
}*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount,
    pub amount_out_min: Option<Uint128>,
    pub paths: Vec<Addr>,
    pub signature: Binary,
    pub recipient: Addr,
    pub current_index: u32,
}
