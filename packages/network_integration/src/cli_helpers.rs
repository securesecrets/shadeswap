use schemars::JsonSchema;
use shadeswap_shared::viewing_keys::ViewingKey;
use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::cli_types::StoredContract;
use secretcli::secretcli::{init, handle, Report};
use secretcli::{cli_types::NetContract, secretcli::query};
use serde::{Serialize, Deserialize};
use snip20_reference_impl::contract;
use std::fmt::Display;
use std::fs;
use cosmwasm_std::{
    Binary, HumanAddr, Uint128, Env
};
use shadeswap_shared::{
    amm_pair::{AMMPair},
    msg::{
        amm_pair::{
            HandleMsg as AMMPairHandlMsg,
        },
        factory::{
            HandleMsg as FactoryHandleMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse, InitMsg as FactoryInitMsg
        },
        router::{
            HandleMsg as RouterHandleMsg,
        },
    },
    stake_contract::StakingContractInit,
    Pagination, TokenPair, TokenPairAmount, TokenType,
};
use shadeswap_shared::snip20_reference_impl::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg, InitialBalance,
};

use serde_json::Result;
// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";
pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";

pub const STORE_GAS: &str = "10000000";
pub const GAS: &str = "800000";
pub const VIEW_KEY: &str = "password";
pub const ACCOUNT_KEY: &str = "a";
pub const STAKER_KEY: &str = "b";
pub const SHADE_DAO_KEY: &str = "c";

pub fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

pub fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

pub fn print_contract(contract: &NetContract) {
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

pub fn print_stored_contract(contract: &StoredContract) {
    println!(
        "\tID: {}\n\tHash: {}",
        contract.id, contract.code_hash
    );
}

pub fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

pub fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

pub fn store_struct<T: serde::Serialize>(path: &str, data: &T) {
    fs::write(
        path,
        serde_json::to_string_pretty(data).expect("Could not serialize data"),
    )
    .expect(&format!("Could not store {}", path));
}

/// This type represents optional configuration values which can be overridden.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
    /// Indicates whether deposit functionality should be enabled
    /// default: False
    pub enable_deposit: Option<bool>,
    /// Indicates whether redeem functionality should be enabled
    /// default: False
    pub enable_redeem: Option<bool>,
    /// Indicates whether mint functionality should be enabled
    /// default: False
    pub enable_mint: Option<bool>,
    /// Indicates whether burn functionality should be enabled
    /// default: False
    pub enable_burn: Option<bool>,
}

impl InitConfig {
    pub fn public_total_supply(&self) -> bool {
        self.public_total_supply.unwrap_or(false)
    }

    pub fn deposit_enabled(&self) -> bool {
        self.enable_deposit.unwrap_or(false)
    }

    pub fn redeem_enabled(&self) -> bool {
        self.enable_redeem.unwrap_or(false)
    }

    pub fn mint_enabled(&self) -> bool {
        self.enable_mint.unwrap_or(false)
    }

    pub fn burn_enabled(&self) -> bool {
        self.enable_burn.unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitialAllowance {
    pub owner: HumanAddr,
    pub spender: HumanAddr,
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

impl InitMsg {
    pub fn config(&self) -> InitConfig {
        self.config.clone().unwrap_or_default()
    }
}


pub fn init_snip20(
    name: String,
    symbol: String, 
    decimals: u8,
    config: Option<InitConfig>,
    reports: &mut Vec<Report>,
    account_key: &str,
    customizedSnip20File: Option<&str>,
    backend: &str
) -> Result<(InitMsg, NetContract)> {
    
    let init_msg = InitMsg {
        name: name.to_string(),
        admin: None,
        symbol: symbol.to_string(),
        decimals: decimals,
        initial_balances: None,
        prng_seed: Default::default(),
        config: config
    };
     
    let snip_20 = init(
        &init_msg,
        customizedSnip20File.unwrap_or(SNIP20_FILE),
        &*generate_label(8),
        account_key,
        Some(STORE_GAS),
        Some(GAS),
        Some(backend),
        reports,
    )?;
    Ok((init_msg, snip_20))
}



pub fn create_viewing_key(env: &Env, seed: Binary, entroy: Binary) -> ViewingKey {
    ViewingKey::new(&env, seed.as_slice(), entroy.as_slice())
}