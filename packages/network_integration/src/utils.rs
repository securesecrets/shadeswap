use colored::*;
use cosmwasm_std::{to_binary, Addr, Binary, Env, MessageInfo, Uint128};
use rand::{distributions::Alphanumeric, Rng};
use schemars::JsonSchema;
use secretcli::cli_types::NetContract;
use secretcli::cli_types::StoredContract;
use secretcli::secretcli::{init, Report};
use serde::{Deserialize, Serialize};
use shadeswap_shared::core::{Callback, ViewingKey};
use shadeswap_shared::snip20::InitialBalance;
use std::fmt::Display;
use std::fs;

use serde_json::Result;
use shadeswap_shared::{
    msg::factory::InitMsg as FactoryInitMsg,
};
// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";
pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";
pub const ADMIN_FILE: &str = "../../misc/admin.wasm.gz";

pub const STORE_GAS: &str = "100000000";
pub const GAS: &str = "8000000";
pub const VIEW_KEY: &str = "password";
pub const API_KEY: &str = "api_key";
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
    println!("\tID: {}\n\tHash: {}", contract.id, contract.code_hash);
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
    pub owner: Addr,
    pub spender: Addr,
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub initial_allowances: Option<Vec<InitialAllowance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
    pub callback: Option<Callback>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsgSnip20 {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

pub fn init_snip20(
    name: String,
    symbol: String,
    decimals: u8,
    config: Option<InitConfig>,
    reports: &mut Vec<Report>,
    account_key: &str,
    customized_snip20_file: Option<&str>,
) -> Result<(InitMsg, NetContract)> {
    let init_msg = InitMsg {
        name: name.to_string(),
        admin: None,
        symbol: symbol.to_string(),
        decimals: decimals,
        initial_balances: None,
        prng_seed: Default::default(),
        config: config,
        initial_allowances: None,
        callback: None,
    };

    let s_token = init(
        &init_msg,
        customized_snip20_file.unwrap_or(SNIP20_FILE),
        &*generate_label(8),
        account_key,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;
    Ok((init_msg, s_token))
}

pub fn init_snip20_cli(
    name: String,
    symbol: String,
    decimals: u8,
    config: Option<InitConfig>,
    reports: &mut Vec<Report>,
    account_key: &str,
    customized_snip20_file: Option<&str>,
    backend: &str,
) -> Result<(InstantiateMsgSnip20, NetContract)> {
    let init_msg = InstantiateMsgSnip20 {
        name: name.to_string(),
        admin: None,
        symbol: symbol.to_string(),
        decimals: decimals,
        initial_balances: None,
        prng_seed: to_binary(&"".to_string()).unwrap(),
        config: config,
    };

    let s_token = init(
        &init_msg,
        customized_snip20_file.unwrap_or(SNIP20_FILE),
        &*generate_label(8),
        account_key,
        Some(STORE_GAS),
        Some(GAS),
        Some(backend),
        reports,
    )?;
    Ok((init_msg, s_token))
}

pub fn create_viewing_key(env: &Env, info: &MessageInfo, seed: Binary, entroy: Binary) -> String {
    ViewingKey::new(&env, info, seed.as_slice(), entroy.as_slice()).to_string()
}

pub fn init_contract_factory(
    account_name: &str,
    backend: &str,
    file_path: &str,
    msg: &FactoryInitMsg,
    reports: &mut Vec<Report>,
) -> Result<NetContract> {
    let contract = init(
        &msg,
        file_path,
        &*generate_label(8),
        account_name,
        Some(STORE_GAS),
        Some(GAS),
        Some(backend),
        reports,
    )?;
    Ok(contract)
}
