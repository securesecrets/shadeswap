use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    ACCOUNT_KEY, AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, SNIP20_FILE, STORE_GAS,
    VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, Report, store_and_return_contract},
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
        secret_toolkit::snip20::{Balance, BalanceResponse},
        Callback, ContractInstantiationInfo, ContractLink,
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
fn run_testnet_router() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    let account = account_address(ACCOUNT_KEY)?;
    println!("Using Account: {}", account.blue());
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];


    let s_lp = store_and_return_contract(LPTOKEN20_FILE, ACCOUNT_KEY,Some(STORE_GAS), Some("test"))?;

    /// Initialize sSCRT
    print_header("Initializing sSCRT");

    let (s_sSINIT, s_sCRT) = init_snip20(
        "SSCRT".to_string(),
        "SSCRT".to_string(),
        6,
        Some(Snip20ComposableConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
        &mut reports,
        ACCOUNT_KEY,
        None
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

    let (s_sSHDINIT, s_sSHD) = init_snip20(
        "SSHD".to_string(),
        "SSHD".to_string(),
        6,
        Some(Snip20ComposableConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
        &mut reports,
        ACCOUNT_KEY,
        None
    )?;

    print_contract(&s_sSHD);

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(
            &msg,
            &s_sSHD,
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
    assert_eq!(
        get_balance(&s_sCRT, account.to_string()),
        Uint128(1000000000)
    );

    println!("\n\tDepositing 1000000000uscrt sSHD");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sSHD,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    println!("\n\tInitializing Pair Contract for dynamic code_hash");

    let test_pair = TokenPair::<HumanAddr>(
        TokenType::CustomToken {
            contract_addr: s_sCRT.address.clone().into(),
            token_code_hash: s_sCRT.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: s_sSHD.address.clone().into(),
            token_code_hash: s_sSHD.code_hash.to_string(),
        },
    );
    

    let s_ammPair = store_and_return_contract(AMM_PAIR_FILE, ACCOUNT_KEY,Some(STORE_GAS), Some("test"))?;

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
                address: HumanAddr(String::from(s_sSHD.address.to_string())),
                code_hash: s_sSHD.code_hash.clone(),
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
        &mut reports,
    )?;

    print_contract(&factory_contract);

    print_header("\n\tInitializing New Pair Contract via Factory");
    {
        let msg = FactoryHandleMsg::CreateAMMPair {
            pair: test_pair.clone(),
            entropy: entropy,
        };

        let result = handle(
            &msg,
            &factory_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();

        println!("{}", result.0.input);
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
                    id: s_sSHD.id.clone(),
                    address: s_sSHD.address.clone(),
                    code_hash: s_sSHD.code_hash.to_string(),
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

            assert_eq!(
                get_balance(&s_sCRT, account.to_string()),
                Uint128(1000000000 - 100)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string()),
                Uint128(1000000000 - 100)
            );

            print_header("\n\tInitiating Swap");

            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr(String::from(ammPair.address.0.to_string())),
                    amount: Uint128(10),
                    msg: Some(
                        to_binary(&InvokeMsg::SwapTokens {
                            expected_return: None,
                            to: Some(HumanAddr(account.to_string())),
                            router_link: None,
                            callback_signature: None,
                        })
                        .unwrap(),
                    ),
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

            assert_eq!(
                get_balance(&s_sCRT, account.to_string()),
                Uint128(999999890)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string()),
                Uint128(999999910)
            );
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

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
