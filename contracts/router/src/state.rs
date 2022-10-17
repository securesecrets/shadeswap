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

pub static CONFIG: &[u8] = b"config";
pub static ADDED_TOKEN_LIST: &[u8] = b"added_token_list";
pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub viewing_key: String,
    pub pair_contract_code_hash: String,
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}


pub fn added_tokens_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, ADDED_TOKEN_LIST)
}

pub fn added_tokens_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, ADDED_TOKEN_LIST)
}

pub fn epheral_storage_w(storage: &mut dyn Storage) -> Singleton<CurrentSwapInfo> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn epheral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<CurrentSwapInfo> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount,
    pub amount_out_min: Option<Uint128>,
    pub paths: Vec<Addr>,
    pub signature: Binary,
    pub recipient: Addr,
    pub current_index: u32,
}
