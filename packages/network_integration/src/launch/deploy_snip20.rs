use cosmwasm_std::Binary;
use cosmwasm_std::Env;
use network_integration::utils::InitConfig;
use network_integration::utils::InitMsg;
use shadeswap_shared::viewing_keys::ViewingKey;

use cosmwasm_std::Uint128;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::to_binary;
use colored::Colorize;
use network_integration::utils::{
    generate_label, print_header, print_warning, GAS,
};
use cosmwasm_std::BalanceResponse;
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};

use serde_json::Result;
use shadeswap_shared::core::ContractInstantiationInfo;
use shadeswap_shared::secret_toolkit::snip20::HandleMsg;
use shadeswap_shared::secret_toolkit::snip20::QueryMsg;
use std::env;
use snip20_reference_impl::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

pub const ACCOUNT_KEY: &str = "deployer";
pub const STORE_GAS: &str = "10000000";
pub const SNIP20_FILE: &str = "./compiled/snip20.wasm.gz";
pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: view_key,
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.amount.amount
}

fn main() -> serde_json::Result<()> {
    let mut reports = vec![];
  
    print_header("Storing Snip20 contracts");
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    // print_warning("Storing Snip20 Token");
    // let s_lp = store_and_return_contract(
    //     &SNIP20_FILE, // &SNIP20_FILE.replace("../", ""),
    //     ACCOUNT_KEY,
    //     Some(STORE_GAS),
    //     Some("test"),
    // )?;
    // println!("{}", s_lp.id);

    print_header("Initializing sSCRT");
    let (s_sSINIT, s_sCRT) = init_snip20(
        "USDT".to_string(),
        "USDT".to_string(),
        18,
        Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
        &mut reports,
        ACCOUNT_KEY,
        Some(&SNIP20_FILE),
    )?;

    println!("{}", s_sSINIT.name);
    println!("{}", s_sCRT.address);
    println!("{}", s_sCRT.code_hash);
    return Ok(());
}

pub fn init_snip20(
    name: String,
    symbol: String, 
    decimals: u8,
    config: Option<InitConfig>,
    reports: &mut Vec<Report>,
    account_key: &str,
    customizedSnip20File: Option<&str>
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
  
               

                    
    