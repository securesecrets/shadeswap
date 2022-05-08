use colored::Colorize;
use network_integration::utils::{
    generate_label, print_contract, print_header, print_vec, print_warning, ACCOUNT_KEY,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, SNIP20_FILE, STORE_GAS, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, Report},
};
use serde_json::Result;
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings, Fee},
    fadroma::{
        scrt::{
            from_binary, log, secret_toolkit::snip20, to_binary, Api, BankMsg, Binary, Coin,
            CosmosMsg, Decimal, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
            QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
        },
        Callback, ContractInstantiationInfo, ContractLink, secret_toolkit::snip20::{Balance, BalanceResponse},
    },
    msg::{
        amm_pair::{HandleMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg},
        factory::{
            HandleMsg as FactoryHandleMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
    },
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType,
};
use std::env;

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

#[test]
fn run_testnet() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    let account = account_address(ACCOUNT_KEY)?;
    println!("Using Account: {}", account.blue());
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];
    let mut repo: Vec<Report> = vec![];

    print_header("Initializing LP TOKEN as template");

    let lp_init_msg = Snip20ComposableMsg {
        name: "SHADESWAP Liquidity Provider (LP) token for secret1jqjdazedmt29rmrtw0k3a4m0gxkemywu3py695-secret1jqjdazedmt29rmrtw0k3a4m0gxkemywu3py695".to_string(),
        admin: None,
        symbol: "SHADE-LP".to_string(),
        decimals: 18,
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

    let s_lp = init(
        &lp_init_msg,
        LPTOKEN20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;
    print_contract(&s_lp);

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

    print_header("Initializing sSHD");

    let sshd_init_msg = Snip20ComposableMsg {
        name: "sSHD".to_string(),
        admin: None,
        symbol: "SSHD".to_string(),
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

    let s_sHD = init(
        &sshd_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut repo,
    )?;

    print_contract(&s_sHD);

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(
            &msg,
            &s_sHD,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;
    }

    assert_eq!(get_balance(&s_sCRT, account.to_string()), Uint128(0));

    println!("\n\tDepositing 1000000000uscrt sSCRT");

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
    assert_eq!(get_balance(&s_sCRT, account.to_string()), Uint128(1000000000));

    println!("\n\tDepositing 1000000000uscrt sSHD");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sHD,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    /*let sshd_init_msg = AMMPairInitMsg {
        callback: None,
        pair: todo!(),
        lp_token_contract: todo!(),
        factory_info: todo!(),
        entropy: todo!(),
        prng_seed: todo!(),
        symbol: todo!(),
    };*/

    /*let s_sHD = init(
        Message {},
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;*/

    println!("\n\tInitializing Pair Contract for dynamic code_hash");

    let test_pair = TokenPair::<HumanAddr>(
        TokenType::CustomToken {
            contract_addr: s_sCRT.address.clone().into(),
            token_code_hash: s_sHD.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: s_sHD.address.clone().into(),
            token_code_hash: s_sHD.code_hash.to_string(),
        },
    );

    let seed = to_binary(&"SEED".to_string()).unwrap();
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();
    let amm_pair_init_msg = AMMPairInitMsg {
        pair: test_pair.clone(),
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        factory_info: ContractLink {
            address: HumanAddr(String::from(
                "secret1y45vkh0n6kplaeqw6ratuertapxupz532vxnn3",
            )),
            code_hash: "Test".to_string(),
        },
        prng_seed: seed,
        callback: None,
        entropy: entropy.clone(),
    };

    let s_ammPair = init(
        &amm_pair_init_msg,
        AMM_PAIR_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;
    print_contract(&s_ammPair);

    print_header("\n\tInitializing Factory Contract");

    let seed = match to_binary(&"SEED".to_string()) {
        Ok(it) => it,
        Err(err) => return Ok(()),
    };

    let factory_msg = FactoryInitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: s_ammPair.code_hash.to_string(),
            id: s_ammPair.id.clone().parse::<u64>().unwrap(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(28, 10000),
            shade_dao_fee: Fee::new(2, 10000),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from(s_sHD.address.to_string())),
                code_hash: s_sHD.code_hash.clone(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: seed,
    };

    let factory_contract = init(
        &factory_msg,
        FACTORY_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut repo,
    )?;

    print_contract(&factory_contract);

    print_header("\n\tInitializing New Pair Contract via Factory");
    {
        let msg = FactoryHandleMsg::CreateAMMPair {
            pair: test_pair.clone(),
            entropy: entropy,
        };

        let mut newAMMPairReport = vec![];
        let result = handle(
            &msg,
            &factory_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut newAMMPairReport,
            None,
        )
        .unwrap();

        println!("{}", result.0.input);
        println!("Events: {}", newAMMPairReport.len());
    }

    print_header("\n\tChecking something was done...");
    {
        let msg = FactoryQueryMsg::ListAMMPairs {
            pagination: Pagination {
                start: 0,
                limit: 10,
            },
        };

        let query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
        if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = query {
            assert_eq!(amm_pairs.len(), 1);
            let ammPair = amm_pairs[0].clone();

            print_header("\n\tIncreasing Allowances");
            handle(
                &snip20::HandleMsg::IncreaseAllowance {
                    spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                    amount: Uint128(100),
                    expiration: None,
                    padding: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_sHD.id.clone(),
                    address: s_sHD.address.clone(),
                    code_hash: s_sHD.code_hash.to_string(),
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            handle(
                &snip20::HandleMsg::IncreaseAllowance {
                    spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                    amount: Uint128(100),
                    expiration: None,
                    padding: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_sCRT.id.clone(),
                    address: s_sCRT.address.clone(),
                    code_hash: s_sCRT.code_hash.to_string(),
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            print_header("\n\tAdding Liquidity");
            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: test_pair.clone(),
                        amount_0: Uint128(100),
                        amount_1: Uint128(100),
                    },
                    slippage: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            
            assert_eq!(get_balance(&s_sCRT, account.to_string()), Uint128(1000000000 - 100));
            assert_eq!(get_balance(&s_sHD, account.to_string()), Uint128(1000000000 - 100));

            print_header("\n\tInitiating Swap");
            // handle(
            //     &snip20::HandleMsg::IncreaseAllowance {
            //         spender: HumanAddr(String::from(ammPair.address.0.to_string())),
            //         amount: Uint128(10),
            //         expiration: None,
            //         padding: None,
            //     },
            //     &NetContract {
            //         label: "".to_string(),
            //         id: s_sCRT.id.clone(),
            //         address: s_sCRT.address.clone(),
            //         code_hash: s_sCRT.code_hash.to_string(),
            //     },
            //     ACCOUNT_KEY,
            //     Some(GAS),
            //     Some("test"),
            //     None,
            //     &mut reports,
            //     None,
            // )
            // .unwrap();

            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr(String::from(ammPair.address.0.to_string())),
                    amount: Uint128(10),
                    msg: Some(to_binary(&InvokeMsg::SwapTokens{ expected_return: None, to: Some(HumanAddr(account.to_string())) }).unwrap()),
                    padding: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_sCRT.id.clone(),
                    address: s_sCRT.address.clone(),
                    code_hash: s_sCRT.code_hash.to_string(),
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();            
            
            assert_eq!(get_balance(&s_sCRT, account.to_string()), Uint128(999999890));
            assert_eq!(get_balance(&s_sHD, account.to_string()), Uint128(999999800));
        } else {
            assert!(false, "Query returned unexpected response")
        }

        /*print_header("\n\tInitiating Swap");
        handle(
            &AMMPairHandlMsg::SwapTokens { offer: todo!(), expected_return: todo!(), to: todo!() },
            &NetContract{ label: "".to_string(), id: s_ammPair.id.clone(), address: ammPair.address.0, code_hash: s_ammPair.code_hash.to_string() },
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        );*/
    }

    /*handle(
        &msg,
        &s_sCRT,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        Some("1000000000uscrt"),
        &mut reports,
        None,
    )?;*/

    Ok(())
}

pub fn get_balance(contract: &NetContract, from: String) -> Uint128 {
    let msg = snip20::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: String::from(VIEW_KEY),
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.balance.amount
}

