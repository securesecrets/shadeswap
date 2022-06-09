use colored::*;
use cosmwasm_std::StdResult;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::secretcli::{init, handle, Report};
use secretcli::{cli_types::NetContract, secretcli::query};
use serde::Serialize;
use shadeswap_shared::fadroma::ViewingKey;
use std::fmt::Display;
use std::fs;

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings, Fee},
    fadroma::{
        scrt::{
            from_binary, log, secret_toolkit::snip20, to_binary, Api, BankMsg, Binary, Coin,
            CosmosMsg, Decimal, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
            QueryRequest, QueryResult, StdError, Storage, Uint128, WasmMsg, WasmQuery,
        },
        Callback, ContractInstantiationInfo, ContractLink, secret_toolkit::snip20::{Balance, BalanceResponse},
    },
};

use serde_json::Result;
// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";

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


pub fn init_snip20(
    name: String,
    symbol: String, 
    decimals: u8,
    config: Option<Snip20ComposableConfig>,
    reports: &mut Vec<Report>,
    account_key: &str,
    customizedSnip20File: Option<&str>
) -> Result<(Snip20ComposableMsg, NetContract)> {
    let init_msg = Snip20ComposableMsg {
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

    let s_sToken = init(
        &init_msg,
        customizedSnip20File.unwrap_or(SNIP20_FILE),
        &*generate_label(8),
        account_key,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;
    Ok((init_msg, s_sToken))
}

pub fn create_viewing_key(env: &Env, seed: Binary, entroy: Binary) -> ViewingKey {
    ViewingKey::new(&env, seed.as_slice(), entroy.as_slice())
}
