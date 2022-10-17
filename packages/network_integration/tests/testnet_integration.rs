use colored::Colorize;
use cosmwasm_std::Addr;

use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use network_integration::utils::InitConfig;
use network_integration::utils::ADMIN_FILE;
use network_integration::utils::API_KEY;
use query_authentication::permit::Permit;
use query_authentication::transaction::PermitSignature;
use query_authentication::transaction::PubKey;

use shadeswap_shared::Contract;
use shadeswap_shared::admin::RegistryAction;
use shadeswap_shared::c_std::Binary;
use shadeswap_shared::core::Fee;
use shadeswap_shared::core::TokenAmount;
use shadeswap_shared::core::TokenPair;
use shadeswap_shared::core::TokenPairAmount;
use shadeswap_shared::core::TokenType;
use shadeswap_shared::query_auth::PermitData;
use shadeswap_shared::snip20;
use shadeswap_shared::staking::AuthQuery;

use cosmwasm_std::to_binary;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_warning, ACCOUNT_KEY,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY, STAKER_KEY,
    STAKING_FILE, STORE_GAS, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract},
};
use serde_json::Result;
use shadeswap_shared::staking::QueryData;
use shadeswap_shared::{
    amm_pair::AMMSettings,
    contract_interfaces::admin::InstantiateMsg as AdminInstantiateMsg,
    core::{ContractInstantiationInfo, ContractLink},
    msg::{
        amm_pair::{
            ExecuteMsg as AMMPairHandlMsg, QueryMsg as AMMPairQueryMsg,
            QueryMsgResponse as AMMPairQueryMsgResponse,
        },
        factory::{
            ExecuteMsg as FactoryExecuteMsg, InitMsg as FactoryInitMsg,
            QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse,
        },
        router::{
            ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg,
            QueryMsg as RouterQueryMsg, QueryMsgResponse as RouterQueryResponse,
        },
        staking::{
            ExecuteMsg as StakingMsgHandle, QueryMsg as StakingQueryMsg,
            QueryResponse as StakingQueryMsgResponse, StakingContractInit,
        },
    },
    Pagination,
};

use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128::from(since_the_epoch.as_millis()))
}

// #[test]
fn run_testnet() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;
    let _shade_dao = account_address(SHADE_DAO_KEY)?;
    let _staker_account = account_address(STAKER_KEY)?;
    println!("Using Account: {}", account.blue());

    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];

    // set viewing key for staker
    print_header("\n\t Set Viewing Key for Staker - Staking Contract password");
    print_header(&to_binary(&QueryData {}).unwrap().to_base64());

    type TestPermit = Permit<PermitData>;
    //secretd tx sign-doc file --from a
    let newPermit = TestPermit{
        params: PermitData { data: to_binary(&QueryData {}).unwrap(), key: "0".to_string()},
        chain_id: Some("secretdev-1".to_string()),
        sequence: Some(Uint128::zero()),
        signature: PermitSignature {
            pub_key: PubKey::new(Binary::from_base64(&"A07oJJ9n4TYTnD7ZStYyiPbB3kXOZvqIMkchGmmPRAzf".to_string()).unwrap()),
            signature: Binary::from_base64(&"bct9+cSJF+m51/be9/Bcc1zwfzYdMGzFMUH4VQl8EW9BuDDok6YEGzw6ZQOmu+rGqlFOfMBGybZbgINjD48rVQ==".to_string()).unwrap(),
        },
        account_number: Some(Uint128::zero()),
        memo: Some("".to_string())
    };

    // print_header("Initializing sSCRT");
    // let (s_sSINIT, s_sCRT) = init_snip20(
    //     "SSCRT".to_string(),
    //     "SSCRT".to_string(),
    //     6,
    //     Some(InitConfig {
    //         public_total_supply: Some(true),
    //         enable_deposit: Some(true),
    //         enable_redeem: Some(true),
    //         enable_mint: Some(true),
    //         enable_burn: Some(false),
    //     }),
    //     &mut reports,
    //     ACCOUNT_KEY,
    //     None,
    // )?;
    // print_contract(&s_sCRT);

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
    print_header("\n\tInitializing Admin Contract");

    let admin_msg = AdminInstantiateMsg {
        super_admin: Some("secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03".to_string()),
    };

    let admin_contract = init(
        &admin_msg,
        &ADMIN_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    let admin_register_msg = RegistryAction::RegisterAdmin { 
        user: "secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03".to_string()
    };


    handle(
        &admin_register_msg,
        &admin_contract,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        Some("1000000000000uscrt"),
        &mut reports,
        None,
    )?;

    print_contract(&admin_contract);
    print_header("Initializing sSCRT");
    let (_s_sSINIT, s_sCRT) = init_snip20(
        "SSCRT".to_string(),
        "SSCRT".to_string(),
        6,
        Some(InitConfig {
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
        let msg = snip20::ExecuteMsg::SetViewingKey {
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

    let (_s_sREWARDSNIP20INIT, s_sREWARDSNIP20) = init_snip20(
        "RWSN".to_string(),
        "RWSN".to_string(),
        6,
        Some(InitConfig {
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
        let msg = snip20::ExecuteMsg::SetViewingKey {
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

    let (_s_sSHDINIT, s_sSHD) = init_snip20(
        "SSHD".to_string(),
        "SSHD".to_string(),
        6,
        Some(InitConfig {
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
        let msg = snip20::ExecuteMsg::SetViewingKey {
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
        let msg = snip20::ExecuteMsg::Deposit { padding: None };

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
        Uint128::new(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSCRT");

    {
        let msg = snip20::ExecuteMsg::Deposit { padding: None };

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
        Uint128::new(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSHD");

    {
        let msg = snip20::ExecuteMsg::Deposit { padding: None };

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

        assert_eq!(
            get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(1000000000000)
        );
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
                address: Addr::unchecked(s_sSHD.address.clone()),
                code_hash: s_sSHD.code_hash.clone(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
        api_key: API_KEY.to_string(),
        authenticator: None,
        admin_auth: Contract { address: Addr::unchecked(admin_contract.address.to_string()), 
            code_hash: admin_contract.code_hash.clone()}
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

    print_header("\n\tInitializing Router");

    let router_msg = RouterInitMsg {
        prng_seed: to_binary(&"".to_string()).unwrap(),
        entropy: to_binary(&"".to_string()).unwrap(),
        pair_contract_code_hash: s_ammPair.code_hash.clone(),
        admin_auth: Contract { address: Addr::unchecked(admin_contract.address.to_string()), 
            code_hash: admin_contract.code_hash.clone()}
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

    print_header("\n\tInitializing New Pair Contract (SNIP20/SNIP20) via Factory");

    let test_pair = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(s_sCRT.address.clone()),
            token_code_hash: s_sCRT.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(s_sSHD.address.clone()),
            token_code_hash: s_sSHD.code_hash.to_string(),
        },
    );

    {
        handle(
            &FactoryExecuteMsg::CreateAMMPair {
                pair: test_pair.clone(),
                entropy: entropy,
                // staking_contract: None,
                staking_contract: Some(StakingContractInit {
                    contract_info: ContractInstantiationInfo {
                        code_hash: staking_contract.code_hash.to_string(),
                        id: staking_contract.id.clone().parse::<u64>().unwrap(),
                    },
                    daily_reward_amount: Uint128::new(3450000000000u128),
                    reward_token: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(s_sREWARDSNIP20.address.clone()),
                        token_code_hash: s_sREWARDSNIP20.code_hash.to_string(),
                    },
                }),
                router_contract: Some(ContractLink {
                    address: Addr::unchecked(router_contract.address.clone()),
                    code_hash: router_contract.code_hash.clone(),
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

    let test_native_pair = TokenPair(
        TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(s_sCRT.address.clone()),
            token_code_hash: s_sCRT.code_hash.to_string(),
        },
    );

    {
        handle(
            &FactoryExecuteMsg::CreateAMMPair {
                pair: test_native_pair.clone(),
                entropy: to_binary(&"".to_string()).unwrap(),
                staking_contract: None,
                router_contract: None, // staking_contract: Some(StakingContractInit {
                                       //     contract_info: ContractInstantiationInfo{
                                       //         code_hash: staking_contract.code_hash.to_string(),
                                       //         id: staking_contract.id.clone().parse::<u64>().unwrap(),
                                       //     },
                                       //     amount: Uint128::new(100000u128),
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

            print_header("\n\tIncreasing Allowances");
            handle(
                &snip20::ExecuteMsg::IncreaseAllowance {
                    spender: ammPair.address.to_string(),
                    amount: Uint128::new(10000000000),
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
                &snip20::ExecuteMsg::IncreaseAllowance {
                    spender: ammPair.address.to_string(),
                    amount: Uint128::new(10000000000),
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
                &snip20::ExecuteMsg::IncreaseAllowance {
                    spender: amm_pair_2.address.to_string(),
                    amount: Uint128::new(10000000000),
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
                    address: ammPair.address.to_string(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                staking_contract_msg,
                None,
            )?;
            if let AMMPairQueryMsgResponse::StakingContractInfo { staking_contract } =
                staking_contract_query
            {
                assert_ne!(staking_contract, None);
            }

            print_header("\n\tAdding Liquidity to SNIP20/20 staking contract");
            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: test_pair.clone(),
                        amount_0: Uint128::new(10000000000),
                        amount_1: Uint128::new(10000000000),
                    },
                    expected_return: None,
                    staking: Some(true),
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.to_string(),
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

            print_header("\n\tAdding Liquidity to NATIVE/SNIP20 staking contract");
            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: test_native_pair.clone(),
                        amount_0: Uint128::new(10000000000),
                        amount_1: Uint128::new(10000000000),
                    },
                    expected_return: None,
                    staking: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: amm_pair_2.address.to_string(),
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
                Uint128::new(1000000000000 - 20000000000)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                Uint128::new(1000000000000 - 10000000000)
            );
            print_header("\n\tRegistering Tokens");

            handle(
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: Addr::unchecked(s_sCRT.address.clone()),
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
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: Addr::unchecked(s_sSHD.address.clone()),
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

            {
                let trade_count_info_msg = AMMPairQueryMsg::GetTradeCount {};
                let trade_count_info_query: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    trade_count_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetTradeCount { count } = trade_count_info_query {
                    assert_eq!(count, 0u64);
                } else {
                    panic!("Trade count couldnt pass")
                }
            }

            {
                let trade_count_info_msg = AMMPairQueryMsg::GetTradeHistory {
                    pagination: Pagination {
                        start: 0u64,
                        limit: 10u8,
                    },
                    api_key: API_KEY.to_string(),
                };
                let trade_count_info_query: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    trade_count_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetTradeHistory { data } = trade_count_info_query {
                    assert_eq!(data.len(), 0u32 as usize);
                } else {
                    panic!("Trade count couldnt pass")
                }
            }

            print_header("\n\t 1. - BUY 100 sSHD Initiating sSCRT to sSHD Swap ");
            let old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            let old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(100),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(10)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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
                (old_scrt_balance - Uint128::new(100))
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance + Uint128::new(89))
            );

            {
                let trade_count_info_msg = AMMPairQueryMsg::GetTradeCount {};
                let trade_count_info_query: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    trade_count_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetTradeCount { count } = trade_count_info_query {
                    assert_eq!(count, 1u64);
                } else {
                    panic!("Trade count couldnt pass")
                }
            }

            {
                let trade_count_info_msg = AMMPairQueryMsg::GetTradeHistory {
                    pagination: Pagination {
                        start: 0u64,
                        limit: 10u8,
                    },
                    api_key: API_KEY.to_string(),
                };
                let trade_count_info_query: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    trade_count_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetTradeHistory { data } = trade_count_info_query {
                    assert_eq!(data.len(), 1u32 as usize);
                } else {
                    panic!("Trade count couldnt pass")
                }
            }

            let old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            print_header("\n\t 2 - BUY 50 sSHD Initiating sSCRT to sSHD Swap ");

            handle(
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(50),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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
                old_shd_balance + Uint128::new(44)
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                (old_scrt_balance - Uint128::new(50))
            );

            print_header("\n\t 3 - SELL 2500 sSHD Initiating sSHD to sSCRT Swap ");
            let old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(2500),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance - Uint128::new(2500))
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance + Uint128::new(2249)
            );

            print_header("\n\t 4 - SELL 36500 sSHD Initiating sSHD to sSCRT Swap ");
            let old_shd_balance = get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(36500),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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

            assert_eq!(
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string()),
                (old_shd_balance - Uint128::new(36500))
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance + Uint128::new(32849)
            );

            print_header("\n\t 5 - BUY 25000 sSHD Initiating sSCRT to sSHD Swap ");
            let mut old_shd_balance =
                get_balance(&s_sSHD, account.to_string(), VIEW_KEY.to_string());
            let mut old_scrt_balance =
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());
            handle(
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(25000),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(5)),
                            paths: vec![ammPair.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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
                old_shd_balance + Uint128::new(22500)
            );

            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(
                    &s_sCRT,
                    router_contract.address.to_string(),
                    VIEW_KEY.to_string()
                ),
                Uint128::new(0)
            );
            assert_eq!(
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                (old_scrt_balance - Uint128::new(25000))
            );

            print_header("\n\tInitiating SCRT to sSCRT Swap");
            old_scrt_balance = get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string());

            handle(
                &RouterExecuteMsg::SwapTokensForExact {
                    offer: TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".to_string(),
                        },
                        amount: Uint128::new(100),
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
                &RouterExecuteMsg::SwapTokensForExact {
                    offer: TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".to_string(),
                        },
                        amount: Uint128::new(100),
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
                &snip20::ExecuteMsg::Send {
                    recipient: router_contract.address.to_string(),
                    amount: Uint128::new(100),
                    msg: Some(
                        to_binary(&RouterInvokeMsg::SwapTokensForExact {
                            expected_return: Some(Uint128::new(10)),
                            paths: vec![ammPair.address.clone(), amm_pair_2.address.clone()],
                            recipient: Some(Addr::unchecked(account.to_string())),
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
                get_balance(&s_sCRT, account.to_string(), VIEW_KEY.to_string()),
                old_scrt_balance
            );

            print_header("\n\tGet Estimated Price for AMM Pair");
            let estimated_price_query_msg = AMMPairQueryMsg::GetEstimatedPrice {
                offer: TokenAmount {
                    token: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(s_sCRT.address.clone()),
                        token_code_hash: s_sCRT.code_hash.clone(),
                    },
                    amount: Uint128::new(100),
                },
                exclude_fee: None,
            };
            let estimated_price_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.to_string(),
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
                    address: ammPair.address.to_string(),
                    code_hash: s_ammPair.code_hash.to_string(),
                },
                lp_token_info_msg,
                None,
            )?;

            if let AMMPairQueryMsgResponse::GetPairInfo {
                liquidity_token,
                factory: _,
                pair: _,
                amount_0: _,
                amount_1: _,
                total_liquidity,
                contract_version: _,
            } = lp_token_info_query
            {
                println!(
                    "\n\tLP Token Address {}",
                    liquidity_token.address.to_string()
                );
                print_header("\n\tLP Token Liquidity - 10000000000");
                assert_eq!(total_liquidity, Uint128::new(10000000000));
            }

            let staking_contract_msg = AMMPairQueryMsg::GetStakingContract {};
            let staking_contract_query: AMMPairQueryMsgResponse = query(
                &NetContract {
                    label: "".to_string(),
                    id: s_ammPair.id.clone(),
                    address: ammPair.address.to_string(),
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
                    &snip20::ExecuteMsg::IncreaseAllowance {
                        spender: staking_contract.clone().unwrap().address.to_string(),
                        amount: Uint128::new(1000000000000),
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
                    &snip20::ExecuteMsg::Send {
                        recipient: staking_contract.clone().unwrap().address.to_string(),
                        amount: Uint128::new(100000000000),
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
                        amount: Uint128::new(5000000000),
                        remove_liqudity: Some(true),
                    },
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.clone(),
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
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    lp_token_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetPairInfo {
                    liquidity_token,
                    factory: _,
                    pair: _,
                    amount_0: _,
                    amount_1: _,
                    total_liquidity,
                    contract_version: _,
                } = lp_token_info_query_unstake_a
                {
                    println!(
                        "\n\tLP Token Address {}",
                        liquidity_token.address.to_string()
                    );
                    print_header("\n\tLP Token Liquidity - 5000000000");
                    assert_eq!(total_liquidity.clone(), Uint128::new(5000000000));
                }
                handle(
                    &StakingMsgHandle::Unstake {
                        amount: Uint128::new(50000000),
                        remove_liqudity: Some(true),
                    },
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.clone(),
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
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    lp_token_info_msg,
                    None,
                )?;
                if let AMMPairQueryMsgResponse::GetPairInfo {
                    liquidity_token,
                    factory: _,
                    pair: _,
                    amount_0: _,
                    amount_1: _,
                    total_liquidity,
                    contract_version: _,
                } = lp_token_info_query_unstake_b
                {
                    println!(
                        "\n\tLP Token Address {}",
                        liquidity_token.address.to_string()
                    );
                    print_header("\n\tLP Token Liquidity - 4950000000");
                    assert_eq!(total_liquidity.clone(), Uint128::new(4950000000));
                }

                print_header("\n\tIncreaseAllowance - 500000000 for liqudity ");
                handle(
                    &snip20::ExecuteMsg::IncreaseAllowance {
                        spender: ammPair.address.to_string(),
                        amount: Uint128::new(500000000),
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
                    &snip20::ExecuteMsg::IncreaseAllowance {
                        spender: ammPair.address.to_string(),
                        amount: Uint128::new(500000000),
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
                            amount_0: Uint128::new(500000000),
                            amount_1: Uint128::new(500000000),
                        },
                        expected_return: None,
                        staking: Some(true),
                    },
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
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
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    lp_token_info_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::GetPairInfo {
                    liquidity_token,
                    factory: _,
                    pair: _,
                    amount_0: _,
                    amount_1: _,
                    total_liquidity,
                    contract_version: _,
                } = lp_token_info_query_unstake
                {
                    println!(
                        "\n\tLP Token Address {}",
                        liquidity_token.address.to_string()
                    );
                    print_header("\n\tLP Token Liquidity - 5449999219");
                    assert_eq!(total_liquidity, Uint128::new(5449999219));
                }

                print_header("\n\tSwap Simulation - Buy 540000SSH");
                let swap_simulation_msg = RouterQueryMsg::SwapSimulation {
                    offer: TokenAmount {
                        amount: Uint128::new(540000),
                        token: TokenType::CustomToken {
                            token_code_hash: s_sCRT.code_hash.to_string(),
                            contract_addr: Addr::unchecked(s_sCRT.address.clone()),
                        },
                    },
                    path: vec![ammPair.address.clone()],
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
                    total_fee_amount: _,
                    lp_fee_amount: _,
                    shade_dao_fee_amount: _,
                    result,
                    price: _,
                } = swap_result_response
                {
                    assert_ne!(result.return_amount, Uint128::new(0u128));
                }

                print_header("\n\tGet Shade DAO Info with Admin Address");
                let get_shade_dao_msg = AMMPairQueryMsg::GetShadeDaoInfo {};
                let shade_dao_response: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    get_shade_dao_msg,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::ShadeDAOInfo {
                    shade_dao_address,
                    shade_dao_fee: _,
                    lp_fee: _,
                    admin_auth: _
                } = shade_dao_response
                {
                    assert_ne!(
                        shade_dao_address.to_string(),
                        Addr::unchecked("".to_string()).to_string()
                    )
                }

                print_header("\n\tGet Claimamble Rewards ");
                let get_claims_reward_msg = StakingQueryMsg::WithPermit {
                    permit: newPermit.clone(),
                    query: AuthQuery::GetClaimReward {
                        time: get_current_timestamp().unwrap(),
                    },
                };
                let claims_reward_response: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
                    },
                    get_claims_reward_msg,
                    None,
                )?;

                // if let StakingQueryMsgResponse::ClaimRewards {
                // } = claims_reward_response
                // {
                //     assert_ne!(amount, Uint128::new(0));
                //     assert_eq!(
                //         reward_token.address.to_string(),
                //         s_sREWARDSNIP20.address.clone().to_string()
                //     )
                // }

                print_header("\n\tGet Staking Contract Config Info");
                let get_config_msg = StakingQueryMsg::GetConfig {};
                let config_query_response: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
                    },
                    get_config_msg,
                    None,
                )?;

                if let StakingQueryMsgResponse::Config {
                    reward_token,
                    lp_token: _,
                    daily_reward_amount,
                    amm_pair,
                    admin_auth: _
                } = config_query_response
                {
                    assert_eq!(
                        reward_token.address.to_string(),
                        s_sREWARDSNIP20.address.clone().to_string()
                    );
                    assert_eq!(
                        reward_token.code_hash.to_string(),
                        s_sREWARDSNIP20.code_hash.clone()
                    );
                    assert_eq!(daily_reward_amount, Uint128::new(3450000000000));
                }
                print_header("\n\tGet Estimated LP Token & Total LP Token Liquditiy");
                let get_estimated_lp_token = AMMPairQueryMsg::GetEstimatedLiquidity {
                    deposit: TokenPairAmount {
                        pair: test_pair.clone(),
                        amount_0: Uint128::new(10000000000),
                        amount_1: Uint128::new(10000000000),
                    }
                };
                let estimated_lp_token: AMMPairQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: s_ammPair.id.clone(),
                        address: ammPair.address.to_string(),
                        code_hash: s_ammPair.code_hash.to_string(),
                    },
                    get_estimated_lp_token,
                    None,
                )?;

                if let AMMPairQueryMsgResponse::EstimatedLiquidity {
                    lp_token,
                    total_lp_token,
                } = estimated_lp_token
                {
                    assert_ne!(lp_token, Uint128::new(0));
                    assert_ne!(total_lp_token, Uint128::new(0))
                }
                print_header("\n\tGetStakeLpTokenInfo For Staker");
                let get_stake_lp_token_info = StakingQueryMsg::WithPermit {
                    permit: newPermit,
                    query: AuthQuery::GetStakerLpTokenInfo {},
                };

                let stake_lp_token_info: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
                    },
                    get_stake_lp_token_info,
                    None,
                )?;

                if let StakingQueryMsgResponse::StakerLpTokenInfo {
                    staked_lp_token,
                    total_staked_lp_token,
                } = stake_lp_token_info
                {
                    assert_ne!(staked_lp_token, Uint128::new(0));
                    assert_ne!(total_staked_lp_token, Uint128::new(0))
                }
                /* TO DO FIX
                print_header("\n\tGetRewardTokenBalance");
                let get_balance_reward_token_msg = StakingQueryMsg::WithPermit { permit: newPermit, query: AuthQuery::GetRewardTokenBalance {
                }};

                let balance_reward_token: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
                    },
                    get_balance_reward_token_msg,
                    None,
                )?;*/

                /*if let StakingQueryMsgResponse::RewardTokenBalance {
                    amount,
                    reward_token,
                } = balance_reward_token
                {
                    assert_eq!(
                        reward_token.address.to_string().clone(),
                        s_sREWARDSNIP20.address.clone()
                    );
                    assert_eq!(
                        reward_token.code_hash.clone(),
                        s_sREWARDSNIP20.code_hash.to_string()
                    );
                    assert_ne!(amount, Uint128::new(0));
                }*/
                print_header("\n\t GetStakerRewardTokenBalance for Non Staker");
                // let get_staker_reward_token_balance_msg =
                //     StakingQueryMsg::GetStakerRewardTokenBalance {
                //         key: String::from(VIEW_KEY),
                //         staker: Addr::unchecked(staker_account.to_string()),
                //     };
                // let staker_reward_token_balance: StakingQueryMsgResponse = query(
                //     &NetContract {
                //         label: "".to_string(),
                //         id: "".to_string(),
                //         address: staking_contract.address.to_string(),
                //         code_hash: staking_contract.code_hash.to_string(),
                //     },
                //     get_staker_reward_token_balance_msg,
                //     None,
                // )?;

                // if let StakingQueryMsgResponse::StakerRewardTokenBalance {
                //     reward_amount,
                //     total_reward_liquidity,
                // } = staker_reward_token_balance
                // {
                //     assert_ne!(reward_amount, Uint128::new(0));
                //     assert_ne!(total_reward_liquidity, Uint128::new(0));
                // }

                /*print_header("\n\t GetStakerRewardTokenBalance for Staker");
                let get_staker_reward_token_balance_msg =
                StakingQueryMsg::WithPermit { permit: newPermit.clone(), query: AuthQuery::GetStakerRewardTokenBalance {
                    }};
                let staker_reward_token_balance: StakingQueryMsgResponse = query(
                    &NetContract {
                        label: "".to_string(),
                        id: "".to_string(),
                        address: staking_contract.clone().unwrap().address.to_string(),
                        code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
                    },
                    get_staker_reward_token_balance_msg,
                    None,
                )?;

                if let StakingQueryMsgResponse::StakerRewardTokenBalance {
                    reward_amount,
                    total_reward_liquidity,
                    reward_token,
                } = staker_reward_token_balance
                {
                    assert_ne!(reward_amount, Uint128::new(0));
                    assert_ne!(total_reward_liquidity, Uint128::new(0));
                    assert_eq!(
                        reward_token.address.to_string(),
                        s_sREWARDSNIP20.address.clone()
                    );
                    assert_eq!(
                        reward_token.code_hash.clone(),
                        s_sREWARDSNIP20.code_hash.to_string()
                    );
                }*/
            }
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}


pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = snip20::QueryMsg::Balance {
        address: from,
        key: view_key,
    };

    let balance: snip20::QueryAnswer = query(contract, &msg, None).unwrap();

    if let snip20::QueryAnswer::Balance { amount } = balance {
        return amount;
    }
    Uint128::zero()
}
