use shadeswap_shared::custom_fee::Fee;
use cosmwasm_std::StdResult;
use shadeswap_shared::viewing_keys::ViewingKey;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::Uint128;
use colored::Colorize;
use cosmwasm_std::to_binary;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    ACCOUNT_KEY, AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY,
    SNIP20_FILE, STAKER_KEY, STAKING_FILE, STORE_GAS, VIEW_KEY,
};
use std::{time::{SystemTime, UNIX_EPOCH}};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};
use serde_json::Result;
use shadeswap_shared::{
    secret_toolkit::snip20::{Balance},
    amm_pair::{AMMPair, AMMSettings},
    msg::{
        amm_pair::{HandleMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, QueryMsgResponse as AMMPairQueryMsgResponse ,
             QueryMsg as AMMPairQueryMsg, InvokeMsg},
        factory::{
            HandleMsg as FactoryHandleMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        staking::{ HandleMsg as StakingMsgHandle, QueryResponse as StakingQueryMsgResponse, QueryMsg as StakingQueryMsg},
        router::{
            HandleMsg as RouterHandleMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg, QueryMsg as RouterQueryMsg, QueryMsgResponse as RouterQueryResponse
        },
    },
    stake_contract::StakingContractInit,
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType, fadroma::prelude::{ContractInstantiationInfo, ContractLink},
};
use std::{
    env
};

use snip20_reference_impl::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

use snip20_reference_impl as snip20;

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
        let msg = snip20::msg::HandleMsg::SetViewingKey {
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
        let msg = snip20::msg::HandleMsg::SetViewingKey {
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
        let msg = snip20::msg::HandleMsg::SetViewingKey {
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
        let msg = snip20::msg::HandleMsg::Deposit { padding: None };

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
        let msg = snip20::msg::HandleMsg::Deposit { padding: None };

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
        let msg = snip20::msg::HandleMsg::Deposit { padding: None };

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
                    contract_info: ContractInstantiationInfo{
                        code_hash: staking_contract.code_hash.to_string(),
                        id: staking_contract.id.clone().parse::<u64>().unwrap(),
                    },
                    amount: Uint128(3450000000000u128),
                    reward_token:  TokenType::CustomToken {
                        contract_addr: s_sREWARDSNIP20.address.clone().into(),
                        token_code_hash: s_sREWARDSNIP20.code_hash.to_string(),
                    },
                })
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
                &snip20::msg::HandleMsg::IncreaseAllowance {
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
                &snip20::msg::HandleMsg::IncreaseAllowance {
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
                &snip20::msg::HandleMsg::IncreaseAllowance {
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
                None
            )?;
            if let AMMPairQueryMsgResponse::StakingContractInfo { staking_contract } = staking_contract_query {
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
                viewing_key: Some(VIEW_KEY.to_string()),
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
            let mut old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            let mut old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None,
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
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None
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
            let mut old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None
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
            let mut old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None
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
            let mut old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None,
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
                            denom: "uscrt".to_string()
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
                            denom: "uscrt".to_string()
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
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()), old_scrt_balance
            );

            print_header("\n\tInitiating Multi Leg Swap sSHD > SCRT");
            
            old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());

            handle(
                &snip20::msg::HandleMsg::Send {
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
                    recipient_code_hash: None,
                    memo: None,
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
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()), old_scrt_balance
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
                exclude_fee: None
            };    
            let estimated_price_query: AMMPairQueryMsgResponse = query( 
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.0.clone(),
                    code_hash: s_ammPair.code_hash.to_string(),
                }, 
                estimated_price_query_msg, 
                None
            )?;
            if let AMMPairQueryMsgResponse::EstimatedPrice { estimated_price } = estimated_price_query {
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
                None
            )?;
            
            if let AMMPairQueryMsgResponse::GetPairInfo { 
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
             } = lp_token_info_query {

                println!("\n\tLP Token Address {}", liquidity_token.address.to_string());
                print_header("\n\tLP Token Liquidity - 10000000000");    
                assert_eq!(
                    total_liquidity,
                    Uint128(10000000000)
                );
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
                None
            )?;

           

            if let AMMPairQueryMsgResponse::StakingContractInfo { 
                staking_contract
             } = staking_contract_query {

                println!("\n\tAllowed IncreaseAllowance for reward token - staking contract");  
                // increase allowance for reward token
                handle(
                    &snip20::msg::HandleMsg::IncreaseAllowance {
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
                    &snip20::msg::HandleMsg::Send {
                        recipient: staking_contract.address.clone(),
                        amount: Uint128(100000000000),
                        msg: None,
                        padding: None,
                        recipient_code_hash: None,
                        memo: None,
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

                println!("\n\tUnstake 5000000000LP TOKEN");  

                handle(
                    &StakingMsgHandle::Unstake {
                       amount: Uint128(5000000000),
                       remove_liqudity: Some(true)
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
                let lp_token_info_query_unstake_a: AMMPairQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    }, 
                    lp_token_info_msg, 
                    None
                )?;
                
                if let AMMPairQueryMsgResponse::GetPairInfo { 
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                } = lp_token_info_query_unstake_a {

                    println!("\n\tLP Token Address {}", liquidity_token.address.to_string());
                    print_header("\n\tLP Token Liquidity - 5000000000");    
                    assert_eq!(
                        total_liquidity.clone(),
                        Uint128(5000000000)
                    );
                }
                
                handle(
                    &StakingMsgHandle::Unstake {
                       amount: Uint128(50000000),
                       remove_liqudity: Some(true)
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
                
                let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};    
                let lp_token_info_query_unstake_b: AMMPairQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    }, 
                    lp_token_info_msg, 
                    None
                )?;
                if let AMMPairQueryMsgResponse::GetPairInfo { 
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                } = lp_token_info_query_unstake_b {

                    println!("\n\tLP Token Address {}", liquidity_token.address.to_string());
                    print_header("\n\tLP Token Liquidity - 4950000000");    
                    assert_eq!(
                        total_liquidity.clone(),
                        Uint128(4950000000)
                    );
                }

                print_header("\n\tIncreaseAllowance - 500000000 for liqudity ");
                handle(
                    &snip20::msg::HandleMsg::IncreaseAllowance {
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
                    &snip20::msg::HandleMsg::IncreaseAllowance {
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
                    None
                )?;
                
                if let AMMPairQueryMsgResponse::GetPairInfo { 
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                } = lp_token_info_query_unstake {

                    println!("\n\tLP Token Address {}", liquidity_token.address.to_string());
                    print_header("\n\tLP Token Liquidity - 5449999219");    
                    assert_eq!(
                        total_liquidity,
                        Uint128(5449999219)
                    );
                }    

                print_header("\n\tSwap Simulation - Buy 540000SSH");
                let swap_simulation_msg = RouterQueryMsg::SwapSimulation {
                    offer: TokenAmount {
                        amount: Uint128(540000),
                        token: TokenType::CustomToken {
                            token_code_hash: s_sCRT.code_hash.to_string(),
                            contract_addr: HumanAddr::from(s_sCRT.address.clone()),
                        },
                    },
                    path: vec![HumanAddr::from(ammPair.address.0.clone())],
                };    

                let swap_result_response: RouterQueryResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: router_contract.id.clone(),
                        address: router_contract.address.clone(),
                        code_hash: router_contract.code_hash.to_string(),
                    }, 
                    swap_simulation_msg, 
                    None,
                )?;          
                
                if let RouterQueryResponse::SwapSimulation { 
                    total_fee_amount,
                    lp_fee_amount,
                    shade_dao_fee_amount,
                    result,
                    price
                } = swap_result_response {                  
                    assert_ne!(
                        result.return_amount,
                        Uint128(0u128)
                    );
                }    

                print_header("\n\tGet Shade DAO Info with Admin Address");
                let get_shade_dao_msg = AMMPairQueryMsg::GetShadeDaoInfo {};    
                let shade_dao_response: AMMPairQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    }, 
                    get_shade_dao_msg, 
                    None
                )?;
                
                if let AMMPairQueryMsgResponse::ShadeDAOInfo { 
                  shade_dao_address,
                  shade_dao_fee,
                  admin_address,
                  lp_fee
                } = shade_dao_response {                  
                    assert_ne!(
                        admin_address.to_string(),
                        HumanAddr::default().to_string()
                    );
                    assert_ne!(
                        shade_dao_address.to_string(),
                        HumanAddr::default().to_string()
                    )
                }  
                
                 
                // set viewing key for staker
                print_header("\n\t Set Viewing Key for Staker - Staking Contract password");
                handle(
                    &StakingMsgHandle::SetVKForStaker {
                        key: "password".to_string()
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
                
                print_header("\n\tGet Claimamble Rewards ");                
                let get_claims_reward_msg = StakingQueryMsg::GetClaimReward {
                    staker: HumanAddr::from(account.to_string()), 
                    key: "password".to_string(),
                    time: get_current_timestamp().unwrap(), 
                };   
                let claims_reward_response: StakingQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    }, 
                    get_claims_reward_msg, 
                    None
                )?;
                
                if let StakingQueryMsgResponse::ClaimReward { 
                        amount
                } = claims_reward_response {                  
                    assert_ne!(
                        amount,
                        Uint128(0)
                    );
                }    

                print_header("\n\tGet Estimated LP Token & Total LP Token Liquditiy");
                let get_estimated_lp_token = AMMPairQueryMsg::GetEstimatedLiquidity {
                    deposit: TokenPairAmount {
                        pair: test_pair.clone(),
                        amount_0: Uint128(10000000000),
                        amount_1: Uint128(10000000000),
                    },
                    slippage: None
                };    
                let estimated_lp_token: AMMPairQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.0.clone(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    }, 
                    get_estimated_lp_token, 
                    None
                )?;
                
                if let AMMPairQueryMsgResponse::EstimatedLiquidity { lp_token, total_lp_token }
                     = estimated_lp_token {                  
                    assert_ne!(
                        lp_token,
                        Uint128(0)
                    );
                    assert_ne!(
                        total_lp_token,
                        Uint128(0)
                    )
                }  

                print_header("\n\tGetStakeLpTokenInfo For Staker");
                let get_stake_lp_token_info = StakingQueryMsg::GetStakerLpTokenInfo {
                  key: "password".to_string(),
                  staker: HumanAddr::from(account.to_string()),
                };    
                let stake_lp_token_info: StakingQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    }, 
                    get_stake_lp_token_info, 
                    None
                )?;
                
                if let StakingQueryMsgResponse::StakerLpTokenInfo { staked_lp_token, total_staked_lp_token } 
                     = stake_lp_token_info {                  
                    assert_ne!(
                        staked_lp_token,
                        Uint128(0)
                    );
                    assert_ne!(
                        total_staked_lp_token,
                        Uint128(0)
                    )
                }  

                print_header("\n\tGetRewardTokenBalance");
                let get_balance_reward_token_msg = StakingQueryMsg::GetRewardTokenBalance {
                  key: String::from(VIEW_KEY),
                  address: HumanAddr::from(account.to_string())
                };    
                let balance_reward_token: StakingQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    }, 
                    get_balance_reward_token_msg, 
                    None
                )?;
                
                if let StakingQueryMsgResponse::RewardTokenBalance { amount }  
                     = balance_reward_token {                  
                    assert_ne!(
                        amount,
                        Uint128(0)
                    );
                }  

                print_header("\n\t GetStakerRewardTokenBalance");
                let get_staker_reward_token_balance_msg = StakingQueryMsg::GetStakerRewardTokenBalance {
                  key: String::from(VIEW_KEY),
                  staker: HumanAddr::from(account.to_string())
                };    
                let staker_reward_token_balance: StakingQueryMsgResponse = query( 
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.address.to_string(),
                        code_hash: staking_contract.code_hash.to_string(),
                    }, 
                    get_staker_reward_token_balance_msg, 
                    None
                )?;
                
                if let StakingQueryMsgResponse::StakerRewardTokenBalance { reward_amount, total_reward_liquidity }   
                     = staker_reward_token_balance {                  
                    assert_ne!(
                        reward_amount,
                        Uint128(0)
                    );
                    assert_ne!(
                        total_reward_liquidity,
                        Uint128(0)
                    );
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
        FACTORY_FILE,
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
                    address: HumanAddr(String::from(factory_contract.address)),
                    code_hash: factory_contract.code_hash,
                },
                entropy: to_binary(&"".to_string()).unwrap(),
                viewing_key: Some(VIEW_KEY.to_string()),
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
                

        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}

pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = snip20::msg::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: view_key,
    };

    let balance: snip20::msg::QueryAnswer = query(contract, &msg, None).unwrap();

    if let snip20::msg::QueryAnswer::Balance { amount } = balance {
        return amount;
    }
    Uint128::zero()
}

pub fn get_current_timestamp()-> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128(since_the_epoch.as_millis()))
}