use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    ACCOUNT_KEY, AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY,
    SNIP20_FILE, STAKER_KEY, STAKING_FILE, STORE_GAS, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
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
        Callback, ContractInstantiationInfo, ContractLink, StakingQuery, ViewingKey,
    },
    msg::{
        amm_pair::{
            HandleMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg,
            QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryMsgResponse,
        },
        factory::{
            HandleMsg as FactoryHandleMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        router::{
            HandleMsg as RouterHandleMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg,
        },
        staking::{
            HandleMsg as StakingMsgHandle, QueryMsg as StakingQueryMsg,
            QueryResponse as StakingQueryMsgResponse,
        },
    },
    stake_contract::StakingContractInit,
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType,
};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

#[test]
fn run_testnet() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;
    let shade_dao = account_address(SHADE_DAO_KEY)?;

    println!("Using Account: {}", account.blue());

    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];

    print_header("Storing all contracts");
    print_warning("Storing LP Token Contract");
    let s_lp =
        store_and_return_contract(LPTOKEN20_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_warning("Storing AMM Pair Token Contract");
    let s_ammPair =
        store_and_return_contract(AMM_PAIR_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_warning("Storing Staking Contract");
    let staking_contract =
        store_and_return_contract(STAKING_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;

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
        None,
    )?;

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

    print_contract(&s_sCRT);

    print_header("Initializing s_sREWARDSNIP20");

    let (s_sREWARDSNIP20INIT, s_sREWARDSNIP20) = init_snip20(
        "RWSN".to_string(),
        "RWSN".to_string(),
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
        None,
    )?;

    print_contract(&s_sREWARDSNIP20);
    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(
            &msg,
            &s_sREWARDSNIP20,
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
        None,
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

    println!("\n\tDepositing 1000000000000uscrt s_sREWARDSNIP20");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sREWARDSNIP20,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    assert_eq!(
        get_balance(&s_sREWARDSNIP20, account.to_string(), VIEW_KEY.to_string()),
        Uint128(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSCRT");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    assert_eq!(
        get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
        Uint128(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSHD");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sSHD,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    print_header("\n\tInitializing Factory Contract");

    let factory_msg = FactoryInitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: s_ammPair.code_hash.to_string(),
            id: s_ammPair.id.clone().parse::<u64>().unwrap(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(8, 100),
            shade_dao_fee: Fee::new(2, 100),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from(shade_dao.to_string())),
                code_hash: s_sSHD.code_hash.clone(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
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

    print_header("\n\tInitializing New Pair Contract (SNIP20/SNIP20) via Factory");

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

    {
        handle(
            &FactoryHandleMsg::CreateAMMPair {
                pair: test_pair.clone(),
                entropy: entropy,
                // staking_contract: None,
                staking_contract: Some(StakingContractInit {
                    contract_info: ContractInstantiationInfo {
                        code_hash: staking_contract.code_hash.to_string(),
                        id: staking_contract.id.clone().parse::<u64>().unwrap(),
                    },
                    amount: Uint128(3450000000000u128),
                    reward_token: TokenType::CustomToken {
                        contract_addr: s_sREWARDSNIP20.address.clone().into(),
                        token_code_hash: s_sREWARDSNIP20.code_hash.to_string(),
                    },
                }),
            },
            &factory_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();
    }

    print_header("\n\tInitializing New Pair Contract (SCRT/SNIP20) via Factory");

    let test_native_pair = TokenPair::<HumanAddr>(
        TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        TokenType::CustomToken {
            contract_addr: s_sCRT.address.clone().into(),
            token_code_hash: s_sCRT.code_hash.to_string(),
        },
    );

    {
        handle(
            &FactoryHandleMsg::CreateAMMPair {
                pair: test_native_pair.clone(),
                entropy: to_binary(&"".to_string()).unwrap(),
                staking_contract: None,
                // staking_contract: Some(StakingContractInit {
                //     contract_info: ContractInstantiationInfo{
                //         code_hash: staking_contract.code_hash.to_string(),
                //         id: staking_contract.id.clone().parse::<u64>().unwrap(),
                //     },
                //     amount: Uint128(100000u128),
                //     reward_token:  TokenType::CustomToken {
                //         contract_addr: s_sCRT.address.clone().into(),
                //         token_code_hash: s_sCRT.code_hash.to_string(),
                //     },
                // })
            },
            &factory_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();
    }

    print_header("\n\tGetting Pairs from Factory");
    {
        let msg = FactoryQueryMsg::ListAMMPairs {
            pagination: Pagination {
                start: 0,
                limit: 10,
            },
        };

        let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
        if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
            assert_eq!(amm_pairs.len(), 2);
            let ammPair = amm_pairs[0].clone();
            let amm_pair_2 = amm_pairs[1].clone();

            print_header("\n\tAdding Liquidity to Pair Contract");
            handle(
                &snip20::HandleMsg::IncreaseAllowance {
                    spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                    amount: Uint128(10000000000),
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
                    amount: Uint128(10000000000),
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

            handle(
                &snip20::HandleMsg::IncreaseAllowance {
                    spender: HumanAddr(String::from(amm_pair_2.address.0.to_string())),
                    amount: Uint128(10000000000),
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

            print_header("\n\tGet Staking Contract");
            let staking_contract_msg = AMMPairQueryMsg::GetStakingContract {};
            let staking_contract_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                staking_contract_msg,
                None,
            )?;
            if let AMMPairQueryMsgResponse::StakingContractInfo { staking_contract } =
                staking_contract_query
            {
                assert_ne!(staking_contract.address, HumanAddr::default());
            }

            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: test_pair.clone(),
                        amount_0: Uint128(10000000000),
                        amount_1: Uint128(10000000000),
                    },
                    slippage: None,
                    staking: Some(true),
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

            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: test_native_pair.clone(),
                        amount_0: Uint128(10000000000),
                        amount_1: Uint128(10000000000),
                    },
                    slippage: None,
                    staking: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: amm_pair_2.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("10000000000uscrt"),
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                Uint128(1000000000000 - 20000000000)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                Uint128(1000000000000 - 10000000000)
            );

            print_header("\n\tInitializing Router");

            let router_msg = RouterInitMsg {
                prng_seed: to_binary(&"".to_string()).unwrap(),
                factory_address: ContractLink {
                    address: HumanAddr(String::from(factory_contract.address)),
                    code_hash: factory_contract.code_hash,
                },
                entropy: to_binary(&"".to_string()).unwrap(),
                viewing_key: Some(ViewingKey::from(VIEW_KEY)),
            };

            let router_contract = init(
                &router_msg,
                ROUTER_FILE,
                &*generate_label(8),
                ACCOUNT_KEY,
                Some(STORE_GAS),
                Some(GAS),
                Some("test"),
                &mut reports,
            )?;
            print_contract(&router_contract);
            print_header("\n\tRegistering Tokens");

            handle(
                &RouterHandleMsg::RegisterSNIP20Token {
                    token: HumanAddr::from(s_sCRT.address.clone()),
                    token_code_hash: s_sCRT.code_hash.to_string(),
                },
                &router_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            handle(
                &RouterHandleMsg::RegisterSNIP20Token {
                    token: HumanAddr::from(s_sSHD.address.clone()),
                    token_code_hash: s_sSHD.code_hash.to_string(),
                },
                &router_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            print_header("\n\t 1. - BUY 100 sSHD Initiating sSCRT to sSHD Swap ");
            let mut old_scrt_balance =
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            let mut old_shd_balance =
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(100),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(10)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sCRT,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                (old_scrt_balance - Uint128(100)).unwrap()
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance + Uint128(89))
            );

            let mut old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            print_header("\n\t 2 - BUY 50 sSHD Initiating sSCRT to sSHD Swap ");

            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(50),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sCRT,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                old_shd_balance + Uint128(44)
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                (old_scrt_balance - Uint128(50)).unwrap()
            );

            print_header("\n\t 3 - SELL 2500 sSHD Initiating sSHD to sSCRT Swap ");
            let mut old_shd_balance =
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance =
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(2500),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sSHD,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance - Uint128(2500)).unwrap()
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance + Uint128(2249)
            );

            print_header("\n\t 4 - SELL 36500 sSHD Initiating sSHD to sSCRT Swap ");
            let mut old_shd_balance =
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance =
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(36500),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sSHD,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance - Uint128(36500)).unwrap()
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance + Uint128(32849)
            );

            print_header("\n\t 5 - BUY 25000 sSHD Initiating sSCRT to sSHD Swap ");
            let mut old_shd_balance =
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance =
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(25000),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sCRT,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                old_shd_balance + Uint128(22500)
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                (old_scrt_balance - Uint128(25000)).unwrap()
            );

            print_header("\n\tInitiating SCRT to sSCRT Swap");
            old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());

            handle(
                &RouterHandleMsg::SwapTokensForExact {
                    offer: TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".to_string(),
                        },
                        amount: Uint128(100),
                    },
                    expected_return: None,
                    path: vec![amm_pair_2.address.clone()],
                    recipient: None,
                },
                &router_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("100uscrt"),
                &mut reports,
                None,
            )
            .unwrap();

            assert!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()) > old_scrt_balance
            );

            print_header("\n\tInitiating Multi Leg Swap SCRT > sSHD");
            old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());

            handle(
                &RouterHandleMsg::SwapTokensForExact {
                    offer: TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".to_string(),
                        },
                        amount: Uint128(100),
                    },
                    expected_return: None,
                    path: vec![amm_pair_2.address.clone(), ammPair.address.clone()],
                    recipient: None,
                },
                &router_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("100uscrt"),
                &mut reports,
                None,
            )
            .unwrap();

            assert!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()) > old_shd_balance
            );

            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance
            );

            print_header("\n\tInitiating Multi Leg Swap sSHD > SCRT");
            old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());

            handle(
                &snip20::HandleMsg::Send {
                    recipient: HumanAddr::from(router_contract.address.to_string()),
                    amount: Uint128(100),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128(10)),
                            paths: vec![ammPair.address.clone(), amm_pair_2.address.clone()],
                            recipient: Some(HumanAddr::from(account.to_string())),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                },
                &s_sSHD,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            assert!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()) < old_shd_balance
            );

            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance
            );

            print_header("\n\tGet Estimated Price for AMM Pair");
            let estimated_price_query_msg = AMMPairQueryMsg::GetEstimatedPrice {
                offer: TokenAmount {
                    token: TokenType::CustomToken {
                        contract_addr: s_sCRT.address.clone().into(),
                        token_code_hash: s_sCRT.code_hash.clone(),
                    },
                    amount: Uint128(100),
                },
            };
            let estimated_price_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                estimated_price_query_msg,
                None,
            )?;
            if let AMMPairQueryMsgResponse::EstimatedPrice { estimated_price } =
                estimated_price_query
            {
                assert_eq!(estimated_price, "0.9".to_string());
            }

            print_header("\n\tGet LP Token for AMM Pair");
            let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
            let lp_token_info_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                lp_token_info_msg,
                None,
            )?;

            if let AMMPairQueryMsgResponse::GetPairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
            } = lp_token_info_query
            {
                println!(
                    "\n\tLP Token Address {}",
                    liquidity_token.address.to_string()
                );
                print_header("\n\tLP Token Liquidity - 10000000000");
                assert_eq!(total_liquidity, Uint128(10000000000));
            }

            let staking_contract_msg = AMMPairQueryMsg::GetStakingContract {};
            let staking_contract_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                staking_contract_msg,
                None,
            )?;

            if let AMMPairQueryMsgResponse::StakingContractInfo { staking_contract } =
                staking_contract_query
            {
                println!("\n\tAllowed IncreaseAllowance for reward token - staking contract");
                // increase allowance for reward token
                handle(
                    &snip20::HandleMsg::IncreaseAllowance {
                        spender: staking_contract.address.clone(),
                        amount: Uint128(1000000000000),
                        expiration: None,
                        padding: None,
                    },
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: s_sREWARDSNIP20.address.to_string(),
                        code_hash: s_sREWARDSNIP20.code_hash.to_string(),
                    },
                    ACCOUNT_KEY,
                    Some(GAS),
                    Some("test"),
                    None,
                    &mut reports,
                    None,
                )
                .unwrap();

                // send Reward token to staking contract
                handle(
                    &snip20::HandleMsg::Send {
                        recipient: staking_contract.address.clone(),
                        amount: Uint128(100000000000),
                        msg: None,
                        padding: None,
                    },
                    &s_sREWARDSNIP20,
                    ACCOUNT_KEY,
                    Some(GAS),
                    Some("test"),
                    None,
                    &mut reports,
                    None,
                )
                .unwrap();

                //Query rewards
                let rewards_query: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    },
                    StakingQueryMsg::GetClaimReward {
                        time: get_current_timestamp().unwrap(),
                        staker: HumanAddr::from(account),
                    },
                    None,
                )?;

                if let StakingQueryMsgResponse::ClaimReward { amount } = rewards_query {
                    assert_ne!(amount, Uint128::zero())
                }

                println!("\n\tUnstake 5000000000LP TOKEN");

                handle(
                    &StakingMsgHandle::Unstake {
                        amount: Uint128(5000000000),
                        remove_liqudity: Some(true),
                    },
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    },
                    ACCOUNT_KEY,
                    Some(GAS),
                    Some("test"),
                    None,
                    &mut reports,
                    None,
                )
                .unwrap();
                print_header("\n\tGet LP Token for AMM Pair");
                let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
                let lp_token_info_query_unstake: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    lp_token_info_msg,
                    None,
                )?;
                if let AMMPairQueryMsgResponse::GetPairInfo {
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                } = lp_token_info_query_unstake
                {
                    println!(
                        "\n\tLP Token Address {}",
                        liquidity_token.address.to_string()
                    );
                    print_header("\n\tLP Token Liquidity - 5000000000");
                    assert_eq!(total_liquidity, Uint128(5000000000));
                }
                print_header("\n\tIncreaseAllowance - 500000000 for liqudity ");
                handle(
                    &snip20::HandleMsg::IncreaseAllowance {
                        spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                        amount: Uint128(500000000),
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
                handle(
                    &snip20::HandleMsg::IncreaseAllowance {
                        spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                        amount: Uint128(500000000),
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
                print_header("\n\tAddLiquidityToAMMContract - 500000000 with staking");
                handle(
                    &AMMPairHandlMsg::AddLiquidityToAMMContract {
                        deposit: TokenPairAmount {
                            pair: test_pair.clone(),
                            amount_0: Uint128(500000000),
                            amount_1: Uint128(500000000),
                        },
                        slippage: None,
                        staking: Some(true),
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
                print_header("\n\tGet LP Token for AMM Pair");
                let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
                let lp_token_info_query_unstake: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    lp_token_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetPairInfo {
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                } = lp_token_info_query_unstake
                {
                    println!(
                        "\n\tLP Token Address {}",
                        liquidity_token.address.to_string()
                    );
                    print_header("\n\tLP Token Liquidity - 5499999219");
                    assert_eq!(total_liquidity, Uint128(5499999219));
                }
            }
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}

#[test]
fn run_test_deploy() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;
    let shade_dao = account_address(SHADE_DAO_KEY)?;

    println!("Using Account: {}", account.blue());

    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];

    print_warning("Storing LP Token Contract");
    let s_lp =
        store_and_return_contract(&LPTOKEN20_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_warning("Storing AMM Pair Token Contract");
    let s_ammPair =
        store_and_return_contract(&AMM_PAIR_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_warning("Storing Staking Contract");
    let staking_contract =
        store_and_return_contract(&STAKING_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;

    print_header("\n\tInitializing Factory Contract");

    let factory_msg = FactoryInitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: s_ammPair.code_hash.to_string(),
            id: s_ammPair.id.clone().parse::<u64>().unwrap(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(8, 100),
            shade_dao_fee: Fee::new(2, 100),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from("".to_string())),
                code_hash: "".to_string(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
    };

    let factory_contract = init(
        &factory_msg,
        &FACTORY_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    print_contract(&factory_contract);

    print_header("\n\tGetting Pairs from Factory");
    {
        let msg = FactoryQueryMsg::ListAMMPairs {
            pagination: Pagination {
                start: 0,
                limit: 10,
            },
        };

        let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
        if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
            assert_eq!(amm_pairs.len(), 0);

            print_header("\n\tInitializing Router");

            let router_msg = RouterInitMsg {
                prng_seed: to_binary(&"".to_string()).unwrap(),
                factory_address: ContractLink {
                    address: HumanAddr(String::from(factory_contract.address.to_string())),
                    code_hash: factory_contract.code_hash.to_string(),
                },
                entropy: to_binary(&"".to_string()).unwrap(),
                viewing_key: Some(ViewingKey::from(VIEW_KEY)),
            };

            let router_contract = init(
                &router_msg,
                &ROUTER_FILE,
                &*generate_label(8),
                ACCOUNT_KEY,
                Some(STORE_GAS),
                Some(GAS),
                Some("test"),
                &mut reports,
            )?;
            print_contract(&router_contract);

            //COMMENT FROM HERE ON TO REMOVE THE TOKEN DEPLOYMENT

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
                Some(&SNIP20_FILE),
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

            print_header("Initializing SSHD");
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
                Some(&SNIP20_FILE),
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

            print_header("Initializing s_sREWARDSNIP20");

            let (s_sREWARDSNIP20INIT, s_sREWARDSNIP20) = init_snip20(
                "RWSN".to_string(),
                "RWSN".to_string(),
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
                Some(&SNIP20_FILE),
            )?;

            print_contract(&s_sREWARDSNIP20);
            {
                let msg = snip20::HandleMsg::SetViewingKey {
                    key: String::from(VIEW_KEY),
                    padding: None,
                };

                handle(
                    &msg,
                    &s_sREWARDSNIP20,
                    ACCOUNT_KEY,
                    Some(GAS),
                    Some("test"),
                    None,
                    &mut reports,
                    None,
                )?;
            }

            {
                handle(
                    &FactoryHandleMsg::CreateAMMPair {
                        pair: test_pair.clone(),
                        entropy: entropy,
                        // staking_contract: None,
                        staking_contract: Some(StakingContractInit {
                            contract_info: ContractInstantiationInfo {
                                code_hash: staking_contract.code_hash.to_string(),
                                id: staking_contract.id.clone().parse::<u64>().unwrap(),
                            },
                            amount: Uint128(100000u128),
                            reward_token: TokenType::CustomToken {
                                contract_addr: s_sREWARDSNIP20.address.clone().into(),
                                token_code_hash: s_sREWARDSNIP20.code_hash.to_string(),
                            },
                        }),
                    },
                    &factory_contract,
                    ACCOUNT_KEY,
                    Some(GAS),
                    Some("test"),
                    None,
                    &mut reports,
                    None,
                )
                .unwrap();
            }

            print_header("\n\tGetting Pairs from Factory");
            {
                let msg = FactoryQueryMsg::ListAMMPairs {
                    pagination: Pagination {
                        start: 0,
                        limit: 10,
                    },
                };

                let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
                if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
                    let ammPair = amm_pairs[0].clone();

                    print_header("\n\tAdding Liquidity to Pair Contract");
                    handle(
                        &snip20::HandleMsg::IncreaseAllowance {
                            spender: HumanAddr(String::from(ammPair.address.0.to_string())),
                            amount: Uint128(100000000),
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
                            amount: Uint128(100000000),
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
                    print_header("\n\tGet Staking Contract");
                    let staking_contract_msg = AMMPairQueryMsg::GetStakingContract {};
                    let staking_contract_query: AMMPairQueryMsgResponse = query(
                        &NetContract {
                            label: "".to_string(),
                            id: s_ammPair.id.clone(),
                            address: ammPair.address.0.clone(),
                            code_hash: s_ammPair.code_hash.to_string(),
                        },
                        staking_contract_msg,
                        None,
                    )?;

                    handle(
                        &AMMPairHandlMsg::AddLiquidityToAMMContract {
                            deposit: TokenPairAmount {
                                pair: test_pair.clone(),
                                amount_0: Uint128(100000000),
                                amount_1: Uint128(100000000),
                            },
                            slippage: None,
                            staking: None,
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
                }
            }
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}

pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = snip20::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: view_key,
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.balance.amount
}

pub fn get_current_timestamp() -> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128(since_the_epoch.as_millis()))
}
