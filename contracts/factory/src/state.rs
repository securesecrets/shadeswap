use cosmwasm_std::{
    Binary, Storage, Addr,
};
use cosmwasm_storage::{singleton, singleton_read, Singleton, ReadonlySingleton, ReadonlyBucket, bucket_read, bucket, Bucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings},
    core::{ContractInstantiationInfo, TokenPair, ViewingKey},
    msg::factory::InitMsg, Contract
};

const AMM_PAIRS_KEYS: &[u8] = b"amm_pair_keys";
const AMM_PAIRS: &[u8] = b"amm_pairs";
const TOTAL_AMM_PAIR: &[u8] = b"total_amm_pairs";
const PRNG_KEY: &[u8] = b"prng_seed";
pub static CONFIG: &[u8] = b"config";
pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";
pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings,
    pub lp_token_contract: ContractInstantiationInfo,
    pub api_key: ViewingKey,
    pub authenticator: Option<Contract>,
    pub admin_auth: Contract
}

impl Config {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            pair_contract: msg.pair_contract,
            amm_settings: msg.amm_settings,
            lp_token_contract: msg.lp_token_contract,
            api_key: ViewingKey(msg.api_key),
            authenticator: msg.authenticator,
            admin_auth: msg.admin_auth
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NextPairKey {
    pub pair: TokenPair,
    pub code_hash: String
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

pub fn prng_seed_w(storage: &mut dyn Storage) -> Singleton<Binary> {
    singleton(storage, PRNG_KEY)
}

pub fn prng_seed_r(storage: &dyn Storage) -> ReadonlySingleton<Binary> {
    singleton_read(storage, PRNG_KEY)
}

pub fn ephemeral_storage_w(storage: &mut dyn Storage) -> Singleton<NextPairKey> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn ephemeral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<NextPairKey> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn amm_pairs_w(storage: &mut dyn Storage) -> Bucket<AMMPair> {
    bucket(storage, AMM_PAIRS)
}

pub fn amm_pairs_r(storage: &dyn Storage) -> ReadonlyBucket<AMMPair> {
    bucket_read(storage, AMM_PAIRS)
}

pub fn amm_pair_keys_w(storage: &mut dyn Storage) -> Bucket<Addr> {
    bucket(storage, AMM_PAIRS_KEYS)
}

pub fn amm_pair_keys_r(storage: &dyn Storage) -> ReadonlyBucket<Addr> {
    bucket_read(storage, AMM_PAIRS_KEYS)
}

pub fn total_amm_pairs_w(storage: &mut dyn Storage) -> Singleton<u64> {
    singleton(storage, TOTAL_AMM_PAIR)
}

pub fn total_amm_pairs_r(storage: &dyn Storage) -> ReadonlySingleton<u64> {
    singleton_read(storage, TOTAL_AMM_PAIR)
}
