use colored::Colorize;
use std::env;
use network_integration::{
    utils::{
        generate_label, print_contract, print_header, print_vec, print_warning, ACCOUNT_KEY,
        GAS, SNIP20_FILE, STORE_GAS,
        VIEW_KEY,
    },
};
use secretcli::secretcli::{account_address, init, handle};
use serde_json::Result;
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20
        }
    }
};


use composable_snip20::msg::{{InitMsg as Snip20ComposableMsg, InitConfig as Snip20ComposableConfig}};


#[test]
fn run_testnet() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    
    let account = account_address(ACCOUNT_KEY)?;
    println!("Using Account: {}", account.blue());

    let mut reports = vec![];

    /// Initialize sSCRT
    print_header("Initializing sSCRT");

    let sscrt_init_msg = Snip20ComposableMsg {
        name: "sSCRT".to_string(),
        admin: None,
        symbol: "SSCRT".to_string(),
        decimals: 6,
        initial_balances: None,
        prng_seed: Default::default(),
        config: Some(Snip20ComposableConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
        initial_allowances: None,
        callback: None,
    };

    let s_sCRT = init(
        &sscrt_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;
    print_contract(&s_sCRT);

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;
    }

    println!("\n\tDepositing 1000000000uscrt");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000uscrt"),
            &mut reports,
            None,
        )?;
    }
    Ok(())
}
