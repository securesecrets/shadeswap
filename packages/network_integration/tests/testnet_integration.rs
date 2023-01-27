use colored::Colorize;
use cosmwasm_std::{to_binary, Addr, QueryResponse, Response, Uint128};

use network_integration::{
    cli_commands::{
        amm_pair_lib::{
            add_amm_pairs, add_liquidity, get_lp_liquidity, get_staking_contract,
            list_pair_from_factory, store_staking_contract,
        },
        factory_lib::{create_factory_contract, deposit_snip20, increase_allowance},
        router_lib::create_router_contract,
        snip20_lib::set_viewing_key,
    },
    utils::{
        generate_label, get_balance, get_current_timestamp, init_snip20, print_contract,
        print_header, InitConfig, ACCOUNT_KEY, ADMIN_FILE, AMM_PAIR_FILE, API_KEY, GAS,
        LPTOKEN20_FILE, SHADE_DAO_KEY, STAKER_KEY, STORE_GAS, VIEW_KEY,
    },
};
use query_authentication::{
    permit::Permit,
    transaction::{PermitSignature, PubKey},
};
use shadeswap_shared::{amm_pair::AMMPair, core::Fee, staking::StakingContractInit};

use shadeswap_shared::{
    admin::RegistryAction,
    c_std::Binary,
    contract_interfaces::admin::InstantiateMsg as AdminInstantiateMsg,
    core::{ContractInstantiationInfo, TokenAmount, TokenPair, TokenPairAmount, TokenType},
    msg::{
        amm_pair::{
            ExecuteMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg as AMMInvokeMsg,
            QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryMsgResponse,
        },
        factory::{
            ExecuteMsg as FactoryExecuteMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        router::{
            ExecuteMsg as RouterExecuteMsg, InvokeMsg as RouterInvokeMsg,
            QueryMsg as RouterQueryMsg, QueryMsgResponse as RouterQueryResponse,
        },
        staking::{
            ExecuteMsg as StakingMsgHandle, InvokeMsg as StakingInvokeMsg,
            QueryMsg as StakingQueryMsg, QueryResponse as StakingQueryMsgResponse,
        },
    },
    query_auth::PermitData,
    router::Hop,
    snip20,
    staking::{AuthQuery, QueryData},
    Contract, Pagination,
};

use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract},
};
use serde_json::Result;

#[test]
fn run_testnet() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;
    let _shade_dao = account_address(SHADE_DAO_KEY)?;
    let _staker_account = account_address(STAKER_KEY)?;
    println!("Using Account: {}", account.blue());
    let mut reports = vec![];

    // set viewing key for staker
    print_header("\n\t Set Viewing Key for Staker - Staking Contract password");

    let pair_contract_code_hash =
        store_and_return_contract(AMM_PAIR_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?
            .code_hash;

    type TestPermit = Permit<PermitData>;
    //secretd tx sign-doc file --from a
    let new_permit = TestPermit{
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
        user: "secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03".to_string(),
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
    let (_, scrt_token) = init_snip20(
        "SCRT".to_string(),
        "SCRT".to_string(),
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

    let snip_20_code_hash = scrt_token.code_hash.clone();

    set_viewing_key(VIEW_KEY, &scrt_token, &mut reports, ACCOUNT_KEY, "test").unwrap();

    print_contract(&scrt_token);
    print_header("Initializing reward_token");

    let (_, reward_token) = init_snip20(
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

    print_contract(&reward_token);

    set_viewing_key(VIEW_KEY, &reward_token, &mut reports, ACCOUNT_KEY, "test").unwrap();

    print_header("Initializing sSHD");

    let (_, shd_token) = init_snip20(
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

    print_contract(&shd_token);

    set_viewing_key(VIEW_KEY, &shd_token, &mut reports, ACCOUNT_KEY, "test").unwrap();

    println!("\n\tDepositing 1000000000000uscrt reward_token");

    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &reward_token.address,
        "1000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&reward_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSCRT");

    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &scrt_token.address,
        "1000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(1000000000000)
    );

    println!("\n\tDepositing 1000000000000uscrt sSHD");

    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &shd_token.address,
        "1000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    let token_1 = TokenType::CustomToken {
        contract_addr: Addr::unchecked(scrt_token.address.clone()),
        token_code_hash: snip_20_code_hash.clone(),
    };

    let token_2 = TokenType::CustomToken {
        contract_addr: Addr::unchecked(shd_token.address.clone()),
        token_code_hash: shd_token.code_hash.to_string(),
    };

    let token_native = TokenType::NativeToken {
        denom: "uscrt".to_string(),
    };

    print_header("\n\tInitializing Factory Contract");

    let factory_contract = create_factory_contract(
        ACCOUNT_KEY,
        "test",
        &mut reports,
        API_KEY,
        "password",
        3,
        100,
        8,
        100,
        &_shade_dao,
        "",
        &admin_contract.address.to_string(),
        &admin_contract.code_hash,
        "",
        "",
    )
    .unwrap();

    print_contract(&factory_contract);

    print_header("\n\tInitializing Router");

    let router_contract = create_router_contract(
        admin_contract.code_hash.to_string(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
        &admin_contract.address,
    )
    .unwrap();

    print_header("\n\tInitializing New Pair Contract (SNIP20/SNIP20) via Factory");

    let token_pair_1 = TokenPair(token_1.clone(), token_2.clone());

    let token_pair_2 = TokenPair(token_native.clone(), token_1.clone());

    add_amm_pairs(
        factory_contract.address.clone(),
        factory_contract.code_hash.clone(),
        "test",
        ACCOUNT_KEY,
        scrt_token.address.clone(),
        snip_20_code_hash.clone(),
        shd_token.address.clone(),
        snip_20_code_hash.clone(),
        "seed",
        Some(reward_token.address.to_string()),
        Some(reward_token.code_hash.to_string()),
        Some(3450000000000u128),
        Some(3450000000000u128),
        18u8,
        &mut reports,
        None,
        None,
    )
    .unwrap();

    print_header("\n\tInitializing New Pair Contract (SCRT/SNIP20) via Factory");

    add_amm_pairs(
        factory_contract.address.clone(),
        factory_contract.code_hash.clone(),
        "test",
        ACCOUNT_KEY,
        "".to_string(),
        "".to_string(),
        scrt_token.address.clone(),
        snip_20_code_hash.clone(),
        "seed",
        None,
        None,
        None,
        None,
        18u8,
        &mut reports,
        Some("PAIR_CONTRACT".to_owned() + &factory_contract.address.to_string()),
        Some("LP_TOKEN_A".to_owned() + &factory_contract.address.to_string()),
    )
    .unwrap();

    print_header("\n\tGetting Pairs from Factory");
    let amm_pairs = list_pair_from_factory(factory_contract.address.clone(), 0, 10).unwrap();
    assert_eq!(amm_pairs.len(), 2);
    let amm_pair_1 = amm_pairs[0].clone();
    let amm_pair_2 = amm_pairs[1].clone();

    print_header("\n\tIncreasing Allowances");

    increase_allowance(
        amm_pair_1.address.to_string(),
        Uint128::new(100000000000),
        shd_token.address.clone(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
    )
    .unwrap();
    increase_allowance(
        amm_pair_1.address.to_string(),
        Uint128::new(100000000000),
        scrt_token.address.clone(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
    )
    .unwrap();
    increase_allowance(
        amm_pair_2.address.to_string(),
        Uint128::new(100000000000),
        scrt_token.address.clone(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
    )
    .unwrap();

    print_header("\n\tGet Staking Contract");

    let staking_contract = get_staking_contract(&amm_pair_1.address.to_string()).unwrap();

    assert_ne!(staking_contract, None);

    print_header("\n\tAdding Liquidity to SNIP20/20 staking contract");

    add_liquidity(
        ACCOUNT_KEY,
        "test",
        amm_pair_1.address.to_string(),
        scrt_token.address.clone(),
        snip_20_code_hash.clone(),
        shd_token.address.clone(),
        snip_20_code_hash.clone(),
        Uint128::new(10000000000),
        Uint128::new(10000000000),
        true,
        "",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(1000000000000 - 10000000000)
    );

    print_header("\n\tAdding Liquidity to NATIVE/SNIP20 staking contract");
    handle(
        &AMMPairHandlMsg::AddLiquidityToAMMContract {
            deposit: TokenPairAmount {
                pair: token_pair_2.clone(),
                amount_0: Uint128::new(10000000000),
                amount_1: Uint128::new(10000000000),
            },
            expected_return: None,
            staking: None,
            execute_sslp_virtual_swap: None,
        },
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_2.address.to_string(),
            code_hash: "".to_string(),
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
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(1000000000000 - 20000000000)
    );
    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(1000000000000 - 10000000000)
    );
    print_header("\n\tRegistering Tokens");

    handle(
        &RouterExecuteMsg::RegisterSNIP20Token {
            token_addr: scrt_token.address.clone(),
            token_code_hash: snip_20_code_hash.clone(),
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
            token_addr: shd_token.address.clone(),
            token_code_hash: shd_token.code_hash.to_string(),
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

    print_header("Get Trade Count");

    {
        let trade_count_info_msg = AMMPairQueryMsg::GetTradeCount {};
        let trade_count_info_query: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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

    let simulation_query: AMMPairQueryMsgResponse = query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_1.address.to_string(),
            code_hash: "".to_string(),
        },
        AMMPairQueryMsg::SwapSimulation {
            offer: TokenAmount {
                token: TokenType::CustomToken {
                    contract_addr: Addr::unchecked(scrt_token.clone().address),
                    token_code_hash: scrt_token.clone().code_hash,
                },
                amount: Uint128::new(100000),
            },
            exclude_fee: None,
        },
        None,
    )
    .unwrap();

    let mut test_amount = Uint128::zero();

    if let AMMPairQueryMsgResponse::SwapSimulation {
        total_fee_amount,
        lp_fee_amount,
        shade_dao_fee_amount,
        result,
        price,
    } = simulation_query
    {
        test_amount = result.return_amount;
    } else {
        panic!("Swap simulation could not be parsed")
    }

    let old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    let old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());
    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(100000),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(10)),
                    path: vec![Hop {
                        addr: amm_pair_1.address.to_string(),
                        code_hash: pair_contract_code_hash.clone(),
                    }],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &scrt_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        (old_shd_token_balance + test_amount)
    );
    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        (old_scrt_balance - Uint128::new(100000))
    );

    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );

    {
        let trade_count_info_msg = AMMPairQueryMsg::GetTradeCount {};
        let trade_count_info_query: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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

    let old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());
    let old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    print_header("\n\t 2 - BUY 50 sSHD Initiating sSCRT to sSHD Swap ");

    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(50),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(5)),
                    path: vec![Hop {
                        addr: amm_pair_1.address.to_string(),
                        code_hash: pair_contract_code_hash.clone(),
                    }],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &scrt_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        old_shd_token_balance + Uint128::new(45)
    );

    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        (old_scrt_balance - Uint128::new(50))
    );

    print_header("\n\t 3 - SELL 2500 sSHD Initiating sSHD to sSCRT Swap ");
    let old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());
    let old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(2500),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(5)),
                    path: vec![Hop {
                        addr: amm_pair_1.address.to_string(),
                        code_hash: pair_contract_code_hash.clone(),
                    }],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &shd_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        (old_shd_token_balance - Uint128::new(2500))
    );

    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        old_scrt_balance + Uint128::new(2225)
    );

    print_header("\n\t 4 - SELL 36500 sSHD Initiating sSHD to sSCRT Swap ");
    let old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());
    let old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(36500),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(5)),
                    path: vec![Hop {
                        addr: amm_pair_1.address.to_string(),
                        code_hash: pair_contract_code_hash.clone(),
                    }],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &shd_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        (old_shd_token_balance - Uint128::new(36500))
    );

    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        old_scrt_balance + Uint128::new(32485)
    );

    print_header("\n\t 5 - BUY 25000 sSHD Initiating sSCRT to sSHD Swap ");
    let mut old_shd_token_balance =
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());
    let mut old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(25000),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(5)),
                    path: vec![Hop {
                        addr: amm_pair_1.address.to_string(),
                        code_hash: pair_contract_code_hash.clone(),
                    }],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &scrt_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()),
        old_shd_token_balance + Uint128::new(22251)
    );

    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(
            &scrt_token,
            router_contract.address.to_string(),
            VIEW_KEY.to_string()
        ),
        Uint128::new(0)
    );
    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        (old_scrt_balance - Uint128::new(25000))
    );

    print_header("\n\tInitiating SCRT to sSCRT Swap");
    old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());

    handle(
        &RouterExecuteMsg::SwapTokensForExact {
            offer: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                amount: Uint128::new(100),
            },
            expected_return: None,
            path: vec![Hop {
                addr: amm_pair_2.address.to_string(),
                code_hash: pair_contract_code_hash.clone(),
            }],
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

    assert!(get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()) > old_scrt_balance);

    print_header("\n\tInitiating Multi Leg Swap SCRT > sSHD");
    old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());

    handle(
        &RouterExecuteMsg::SwapTokensForExact {
            offer: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                amount: Uint128::new(100),
            },
            expected_return: None,
            path: vec![
                Hop {
                    addr: amm_pair_2.address.to_string(),
                    code_hash: pair_contract_code_hash.clone(),
                },
                Hop {
                    addr: amm_pair_1.address.to_string(),
                    code_hash: pair_contract_code_hash.clone(),
                },
            ],
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
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()) > old_shd_token_balance
    );

    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        old_scrt_balance
    );

    print_header("\n\tInitiating Multi Leg Swap sSHD > SCRT");
    old_scrt_balance = get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string());
    old_shd_token_balance = get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string());

    handle(
        &snip20::ExecuteMsg::Send {
            recipient: router_contract.address.to_string(),
            amount: Uint128::new(100),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128::new(10)),
                    path: vec![
                        Hop {
                            addr: amm_pair_1.address.to_string(),
                            code_hash: pair_contract_code_hash.clone(),
                        },
                        Hop {
                            addr: amm_pair_2.address.to_string(),
                            code_hash: pair_contract_code_hash.clone(),
                        },
                    ],
                    recipient: Some(account.to_string()),
                })
                .unwrap(),
            ),
            padding: None,
            recipient_code_hash: None,
            memo: None,
        },
        &shd_token,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    assert!(
        get_balance(&shd_token, account.to_string(), VIEW_KEY.to_string()) < old_shd_token_balance
    );

    assert_eq!(
        get_balance(&scrt_token, account.to_string(), VIEW_KEY.to_string()),
        old_scrt_balance
    );

    print_header("\n\tGet Estimated Price for AMM Pair");
    let estimated_price_query_msg = AMMPairQueryMsg::SwapSimulation {
        offer: TokenAmount {
            token: TokenType::CustomToken {
                contract_addr: Addr::unchecked(scrt_token.address.clone()),
                token_code_hash: snip_20_code_hash.clone(),
            },
            amount: Uint128::new(100),
        },
        exclude_fee: None,
    };
    let estimated_price_query: AMMPairQueryMsgResponse = query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_1.address.to_string(),
            code_hash: "".to_string(),
        },
        estimated_price_query_msg,
        None,
    )?;
    if let AMMPairQueryMsgResponse::SwapSimulation {
        total_fee_amount: _,
        lp_fee_amount: _,
        shade_dao_fee_amount: _,
        result: _,
        price,
    } = estimated_price_query
    {
        assert_eq!(price, "0.9".to_string());
    }

    print_header("\n\tGet LP Token for AMM Pair");
    let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
    let lp_token_info_query: AMMPairQueryMsgResponse = query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_1.address.to_string(),
            code_hash: "".to_string(),
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
        fee_info: _,
    } = lp_token_info_query
    {
        println!(
            "\n\tLP Token Address {}",
            liquidity_token.address.to_string()
        );
        print_header("\n\tLP Token Liquidity - 10000000000");
        assert_eq!(total_liquidity, Uint128::new(10000000000));
    }

    let staking_contract_msg = AMMPairQueryMsg::GetConfig {};
    let staking_contract_query: AMMPairQueryMsgResponse = query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_1.address.to_string(),
            code_hash: "".to_string(),
        },
        staking_contract_msg,
        None,
    )?;

    if let AMMPairQueryMsgResponse::GetConfig {
        staking_contract,
        factory_contract,
        lp_token,
        pair,
        custom_fee,
    } = staking_contract_query
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
                address: reward_token.address.to_string(),
                code_hash: reward_token.code_hash.to_string(),
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
            &reward_token,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();

        println!("\n\tUnstake 5000000000LP TOKEN");

        let mut old_total_staked: Uint128;

        let total_currently_staked_msg: StakingQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: staking_contract.clone().unwrap().address.to_string(),
                code_hash: "".to_string(),
            },
            StakingQueryMsg::GetConfig {},
            None,
        )?;

        if let StakingQueryMsgResponse::GetConfig {
            total_staked_lp_token,
            reward_token,
            lp_token,
            amm_pair,
            admin_auth,
        } = total_currently_staked_msg
        {
            old_total_staked = total_staked_lp_token;
        } else {
            panic!("Failed to get correct response")
        }

        handle(
            &StakingMsgHandle::Unstake {
                amount: Uint128::new(5000000000),
                remove_liquidity: Some(true),
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

        let total_currently_staked_msg: StakingQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: staking_contract.clone().unwrap().address.to_string(),
                code_hash: "".to_string(),
            },
            StakingQueryMsg::GetConfig {},
            None,
        )?;

        if let StakingQueryMsgResponse::GetConfig {
            total_staked_lp_token,
            reward_token,
            lp_token,
            amm_pair,
            admin_auth,
        } = total_currently_staked_msg
        {
            println!("{} - {}", old_total_staked, total_staked_lp_token);
            assert!(old_total_staked > total_staked_lp_token);
        } else {
            panic!("Failed to get correct response")
        }

        print_header("\n\tGet LP Token for AMM Pair");
        let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
        let lp_token_info_query_unstake_a: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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
            fee_info: _,
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
                remove_liquidity: Some(true),
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
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
            },
            lp_token_info_msg,
            None,
        )?;
        if let AMMPairQueryMsgResponse::GetPairInfo {
            liquidity_token,
            factory: _,
            pair: _,
            amount_0,
            amount_1,
            total_liquidity,
            contract_version: _,
            fee_info: _,
        } = lp_token_info_query_unstake_b
        {
            println!(
                "\n\tLP Token Address {}",
                liquidity_token.address.to_string()
            );
            print_header(&("\n\tLP Token 0 Amount - ".to_owned() + &amount_0.to_string()));
            print_header(&("\n\tLP Token 1 Amount - ".to_owned() + &amount_1.to_string()));
            print_header("\n\tLP Token Liquidity - 4950000000");
            assert_eq!(total_liquidity.clone(), Uint128::new(4950000000));
        }

        print_header("\n\tIncreaseAllowance - 500000000 for liqudity ");
        handle(
            &snip20::ExecuteMsg::IncreaseAllowance {
                spender: amm_pair_1.address.to_string(),
                amount: Uint128::new(500000000),
                expiration: None,
                padding: None,
            },
            &NetContract {
                label: "".to_string(),
                id: scrt_token.id.clone(),
                address: scrt_token.address.clone(),
                code_hash: snip_20_code_hash.clone(),
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
                spender: amm_pair_1.address.to_string(),
                amount: Uint128::new(500000000),
                expiration: None,
                padding: None,
            },
            &NetContract {
                label: "".to_string(),
                id: shd_token.id.clone(),
                address: shd_token.address.clone(),
                code_hash: shd_token.code_hash.to_string(),
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
                    pair: token_pair_1.clone(),
                    amount_0: Uint128::new(500000000),
                    amount_1: Uint128::new(500000000),
                },
                expected_return: None,
                staking: Some(true),
                execute_sslp_virtual_swap: None,
            },
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
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
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
            },
            lp_token_info_msg,
            None,
        )?;

        if let AMMPairQueryMsgResponse::GetPairInfo {
            liquidity_token,
            factory: _,
            pair: _,
            amount_0,
            amount_1,
            total_liquidity,
            contract_version: _,
            fee_info: _,
        } = lp_token_info_query_unstake
        {
            println!(
                "\n\tLP Token Address {}",
                liquidity_token.address.to_string()
            );
            print_header("\n\tLP Token Liquidity - (5449995639");
            print_header(&("\n\tLP Token 0 Amount - ".to_owned() + &amount_0.to_string()));
            print_header(&("\n\tLP Token 1 Amount - ".to_owned() + &amount_1.to_string()));
            assert_eq!(total_liquidity, Uint128::new(5449995639u128));

            let get_stake_lp_token_info = StakingQueryMsg::WithPermit {
                permit: new_permit.clone(),
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

            let mut old_staked_lp_token = Uint128::zero();
            let mut old_total_staked_lp_token = Uint128::zero();

            if let StakingQueryMsgResponse::GetStakerLpTokenInfo {
                staked_lp_token,
                total_staked_lp_token,
            } = stake_lp_token_info
            {
                old_staked_lp_token = staked_lp_token;
                old_total_staked_lp_token = total_staked_lp_token
            }

            print_header("\n\tRAW Adding Liquidity to SNIP20/20 staking contract");
            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: token_pair_1.clone(),
                        amount_0: Uint128::new(10000000000),
                        amount_1: Uint128::new(10000000000),
                    },
                    expected_return: None,
                    staking: Some(false),
                    execute_sslp_virtual_swap: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: "".to_string(),
                    address: amm_pair_1.address.to_string(),
                    code_hash: "".to_string(),
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
                &snip20::ExecuteMsg::Send {
                    recipient: staking_contract.clone().unwrap().address.to_string(),
                    amount: Uint128::new(1000),
                    msg: Some(
                        to_binary(&StakingInvokeMsg::Stake {
                            from: account.clone(),
                        })
                        .unwrap(),
                    ),
                    padding: None,
                    recipient_code_hash: Some(
                        staking_contract.clone().unwrap().code_hash.to_string(),
                    ),
                    memo: None,
                },
                &NetContract {
                    label: "".to_string(),
                    id: "".to_string(),
                    address: liquidity_token.address.to_string(),
                    code_hash: liquidity_token.code_hash,
                },
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();

            let get_stake_lp_token_info = StakingQueryMsg::WithPermit {
                permit: new_permit.clone(),
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

            if let StakingQueryMsgResponse::GetStakerLpTokenInfo {
                staked_lp_token,
                total_staked_lp_token,
            } = stake_lp_token_info
            {
                assert!(old_staked_lp_token < staked_lp_token);
                assert!(old_total_staked_lp_token < total_staked_lp_token);
            }

            print_header("\n\tEND Adding Liquidity to SNIP20/20 staking contract");
        }

        print_header("\n\tSwap Simulation - Buy 540000SSH");
        let swap_simulation_msg = RouterQueryMsg::SwapSimulation {
            offer: TokenAmount {
                amount: Uint128::new(540000),
                token: TokenType::CustomToken {
                    token_code_hash: snip_20_code_hash.clone(),
                    contract_addr: Addr::unchecked(scrt_token.address.clone()),
                },
            },
            path: vec![Hop {
                addr: amm_pair_1.address.to_string(),
                code_hash: pair_contract_code_hash.clone(),
            }],
            exclude_fee: None,
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
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
            },
            get_shade_dao_msg,
            None,
        )?;

        if let AMMPairQueryMsgResponse::GetShadeDaoInfo {
            shade_dao_address,
            shade_dao_fee: _,
            lp_fee: _,
            admin_auth: _,
        } = shade_dao_response
        {
            assert_ne!(
                shade_dao_address.to_string(),
                Addr::unchecked("".to_string()).to_string()
            )
        }

        print_header("\n\tGet Claimamble Rewards ");
        let get_claims_reward_msg = StakingQueryMsg::WithPermit {
            permit: new_permit.clone(),
            query: AuthQuery::GetClaimReward {
                time: get_current_timestamp().unwrap(),
            },
        };
        let _claims_reward_response: StakingQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: staking_contract.clone().unwrap().address.to_string(),
                code_hash: staking_contract.clone().unwrap().code_hash.to_string(),
            },
            get_claims_reward_msg,
            None,
        )?;

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

        if let StakingQueryMsgResponse::GetConfig {
            reward_token,
            lp_token: _,
            amm_pair: _,
            admin_auth: _,
            total_staked_lp_token,
        } = config_query_response
        {
            assert_eq!(
                reward_token.address.to_string(),
                reward_token.address.clone().to_string()
            );
            assert_eq!(
                reward_token.code_hash.to_string(),
                reward_token.code_hash.clone()
            );
        }
        print_header("\n\tGet Estimated LP Token & Total LP Token Liquditiy");
        let get_estimated_lp_token = AMMPairQueryMsg::GetEstimatedLiquidity {
            deposit: TokenPairAmount {
                pair: token_pair_1.clone(),
                amount_0: Uint128::new(10000000000),
                amount_1: Uint128::new(10000000000),
            },
            sender: Addr::unchecked(account.clone()),
            execute_sslp_virtual_swap: None,
        };
        let estimated_lp_token: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_1.address.to_string(),
                code_hash: "".to_string(),
            },
            get_estimated_lp_token,
            None,
        )?;

        if let AMMPairQueryMsgResponse::GetEstimatedLiquidity {
            lp_token,
            total_lp_token,
        } = estimated_lp_token
        {
            assert_ne!(lp_token, Uint128::new(0));
            assert_ne!(total_lp_token, Uint128::new(0))
        }
        print_header("\n\tGetStakeLpTokenInfo For Staker");
        let get_stake_lp_token_info = StakingQueryMsg::WithPermit {
            permit: new_permit.clone(),
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

        if let StakingQueryMsgResponse::GetStakerLpTokenInfo {
            staked_lp_token,
            total_staked_lp_token,
        } = stake_lp_token_info
        {
            assert_ne!(staked_lp_token, Uint128::new(0));
            assert_ne!(total_staked_lp_token, Uint128::new(0))
        }
    }

    print_header("Running all queries");

    print_header("Pair Contract");

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::SwapSimulation {
                offer: TokenAmount {
                    token: token_1.clone(),
                    amount: Uint128::from(100u64),
                },
                exclude_fee: None
            },
        )?,
        AMMPairQueryMsgResponse::SwapSimulation { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetConfig {},
        )?,
        AMMPairQueryMsgResponse::GetConfig { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetPairInfo {},
        )?,
        AMMPairQueryMsgResponse::GetPairInfo { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetTradeHistory {
                api_key: API_KEY.to_string(),
                pagination: Pagination {
                    limit: 10,
                    start: 0
                }
            }
        )?,
        AMMPairQueryMsgResponse::GetTradeHistory { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetWhiteListAddress {},
        )?,
        AMMPairQueryMsgResponse::GetWhiteListAddress { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetTradeCount {},
        )?,
        AMMPairQueryMsgResponse::GetTradeCount { .. }
    ));

    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetShadeDaoInfo {},
        )?,
        AMMPairQueryMsgResponse::GetShadeDaoInfo { .. }
    ));

    // This will check half of extra = 0
    assert!(matches!(
        test_query_successful(
            amm_pair_1.address.to_string(),
            AMMPairQueryMsg::GetEstimatedLiquidity {
                deposit: TokenPairAmount {
                    pair: token_pair_1.clone(),
                    amount_0: Uint128::from(1000u64),
                    amount_1: Uint128::from(1000u64)
                },
                sender: Addr::unchecked(account),
                execute_sslp_virtual_swap: None,
            },
        )?,
        AMMPairQueryMsgResponse::GetEstimatedLiquidity { .. }
    ));

    print_header("Router");

    assert!(matches!(
        test_query_successful(
            router_contract.address.to_string(),
            RouterQueryMsg::GetConfig {},
        )?,
        RouterQueryResponse::GetConfig { .. }
    ));

    assert!(matches!(
        test_query_successful(
            router_contract.address.to_string(),
            RouterQueryMsg::SwapSimulation {
                offer: TokenAmount {
                    token: token_1.clone(),
                    amount: Uint128::from(100u64),
                },
                path: vec![Hop {
                    addr: amm_pair_1.address.to_string(),
                    code_hash: amm_pair_1.code_hash.to_string(),
                }],
                exclude_fee: None
            }
        )?,
        RouterQueryResponse::SwapSimulation { .. }
    ));

    print_header("Factory Contract");

    assert!(matches!(
        test_query_successful(
            factory_contract.address.to_string(),
            FactoryQueryMsg::GetConfig {},
        )?,
        FactoryQueryResponse::GetConfig { .. }
    ));

    assert!(matches!(
        test_query_successful(
            factory_contract.address.to_string(),
            FactoryQueryMsg::ListAMMPairs {
                pagination: Pagination {
                    start: 0,
                    limit: 10
                }
            },
        )?,
        FactoryQueryResponse::ListAMMPairs { .. }
    ));

    assert!(matches!(
        test_query_successful(
            factory_contract.address.to_string(),
            FactoryQueryMsg::GetAMMPairAddress { pair: token_pair_1 },
        )?,
        FactoryQueryResponse::GetAMMPairAddress { .. }
    ));

    assert!(matches!(
        test_query_successful(
            factory_contract.address.to_string(),
            FactoryQueryMsg::AuthorizeApiKey {
                api_key: API_KEY.to_string()
            },
        )?,
        FactoryQueryResponse::AuthorizeApiKey { .. }
    ));

    print_header("Staking Contract");

    let found_staking_contract = staking_contract.unwrap();
    assert!(matches!(
        test_query_successful(
            found_staking_contract.address.to_string(),
            StakingQueryMsg::GetConfig {},
        )?,
        StakingQueryMsgResponse::GetConfig { .. }
    ));

    assert!(matches!(
        test_query_successful(
            found_staking_contract.address.to_string(),
            StakingQueryMsg::GetRewardTokens {},
        )?,
        StakingQueryMsgResponse::GetRewardTokens { .. }
    ));

    assert!(matches!(
        test_query_successful(
            found_staking_contract.address.to_string(),
            StakingQueryMsg::WithPermit {
                permit: new_permit.clone(),
                query: AuthQuery::GetStakerLpTokenInfo {}
            },
        )?,
        StakingQueryMsgResponse::GetStakerLpTokenInfo { .. }
    ));

    assert!(matches!(
        test_query_successful(
            found_staking_contract.address.to_string(),
            StakingQueryMsg::WithPermit {
                permit: new_permit.clone(),
                query: AuthQuery::GetClaimReward {
                    time: get_current_timestamp().unwrap()
                }
            },
        )?,
        StakingQueryMsgResponse::GetClaimReward { .. }
    ));

    //////////////////////////////////
    /// //////////////////////////////
    // TEST CREATE STANDALONE_AMM_PAIR
    ///////////////////////////////////
    /// ///////////////////////////////
    /// ///////////////////////////////
    let account = account_address(ACCOUNT_KEY)?;
    // CREATE USDT SNIP20
    print_header("Initializing USDT");
    let (_, usdt_token) = init_snip20(
        "USDT".to_string(),
        "USDT".to_string(),
        8,
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
    let snip_20_code_hash = usdt_token.code_hash.clone();
    set_viewing_key(VIEW_KEY, &usdt_token, &mut reports, ACCOUNT_KEY, "test").unwrap();
    print_contract(&usdt_token.clone());
    println!("\n\tDepositing 2000000000000uscrt USDT");
    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &usdt_token.address,
        "2000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(2000000000000)
    );

    // CREATE REWARD TOKEN
    print_header("Initializing Reward_token");
    let (_, reward_token) = init_snip20(
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
    print_contract(&reward_token);
    set_viewing_key(VIEW_KEY, &reward_token, &mut reports, ACCOUNT_KEY, "test").unwrap();
    println!("\n\tDepositing 2000000000000uscrt REWARD TOKEN");
    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &reward_token.address,
        "2000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&reward_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(2000000000000)
    );

    // CREATE ETH SNIP20
    print_header("Initializing ETH");
    let (_, eth_token) = init_snip20(
        "ETH".to_string(),
        "ETH".to_string(),
        8,
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
    print_contract(&eth_token);
    set_viewing_key(VIEW_KEY, &eth_token, &mut reports, ACCOUNT_KEY, "test").unwrap();
    println!("\n\tDepositing 2000000000000uscrt ETH");
    deposit_snip20(
        ACCOUNT_KEY,
        "test",
        &eth_token.address,
        "2000000000000uscrt",
        &mut reports,
    )
    .unwrap();

    assert_eq!(
        get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(2000000000000)
    );

    // STORE STAKING CONTRACT
    let staking_contract = store_staking_contract(ACCOUNT_KEY, "test").unwrap();
    let staking_contract_init: Option<StakingContractInit> = Some(StakingContractInit {
        contract_info: ContractInstantiationInfo {
            code_hash: staking_contract.code_hash.to_string(),
            id: staking_contract.id.clone().parse::<u64>().unwrap(),
        },
        daily_reward_amount: Uint128::new(30000u128),
        reward_token: TokenType::CustomToken {
            contract_addr: Addr::unchecked(reward_token.address.to_string()),
            token_code_hash: reward_token.code_hash.to_string(),
        },
        valid_to: Uint128::new(40000000u128),
        custom_label: Some(("THIS IS A TEST".to_owned() + &factory_contract.address.to_string()).to_string()),
    });

    // STORING LP TOKEN
    let lp_token =
        store_and_return_contract(&LPTOKEN20_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))
            .unwrap();

    // CREATE AMM PAIR USDT-ETH
    let amm_pair_init_msg = AMMPairInitMsg {
        pair: TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(usdt_token.address.to_string()),
                token_code_hash: usdt_token.code_hash.to_string(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(eth_token.address.to_string()),
                token_code_hash: eth_token.code_hash.to_string(),
            },
        ),
        lp_token_contract: ContractInstantiationInfo {
            code_hash: lp_token.code_hash.to_string(),
            id: lp_token.id.parse::<u64>().unwrap(),
        },
        factory_info: None,
        prng_seed: to_binary("seed").unwrap(),
        entropy: to_binary("password").unwrap(),
        admin_auth: Contract {
            address: Addr::unchecked(admin_contract.address.to_string()),
            code_hash: admin_contract.code_hash.to_string(),
        },
        staking_contract: staking_contract_init,
        custom_fee: Some(shadeswap_shared::core::CustomFee {
            shade_dao_fee: Fee::new(8, 100),
            lp_fee: Fee::new(3, 100),
        }),
        arbitrage_contract: None,
        lp_token_decimals: 18u8,
        lp_token_custom_label: Some("THIS IS A TEST 2".to_owned() + &factory_contract.address.to_string()),
    };
    // CREATE AMM PAIR
    let amm_pair_contract = init(
        &amm_pair_init_msg,
        &AMM_PAIR_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    print_contract(&amm_pair_contract);
    print_header("\n\tAMMPAIR CONTRACT ADDRESS ");
    let token_pair = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(usdt_token.address.to_string()),
            token_code_hash: usdt_token.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(eth_token.address.to_string()),
            token_code_hash: eth_token.code_hash.to_string(),
        },
    );
    let add_amm_pair_msg = FactoryExecuteMsg::AddAMMPairs {
        amm_pairs: vec![AMMPair {
            pair: token_pair.clone(),
            address: Addr::unchecked(amm_pair_contract.address.to_string()),
            code_hash: amm_pair_contract.code_hash.to_string(),
            enabled: true,
        }],
    };

    let _ = handle(
        &add_amm_pair_msg,
        &factory_contract,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )?;

    print_header("\n\tGetting New Pairs from Factory");
    let amm_pairs = list_pair_from_factory(factory_contract.address.clone(), 0, 10).unwrap();
    assert_eq!(amm_pairs.len(), 3);
    assert_eq!(
        amm_pairs[2].address.to_string(),
        amm_pair_contract.address.to_string()
    );

    print_header("\n\tIncrease Allowance for New AMM Pair");
    increase_allowance(
        amm_pair_contract.address.to_string(),
        Uint128::new(500000000000),
        usdt_token.address.clone(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
    )
    .unwrap();
    increase_allowance(
        amm_pair_contract.address.to_string(),
        Uint128::new(500000000000),
        eth_token.address.clone(),
        ACCOUNT_KEY,
        "test",
        &mut reports,
    )
    .unwrap();

    print_header("\n\tAdding Liquidity to USDT/ETH staking contract");
    handle(
        &AMMPairHandlMsg::AddLiquidityToAMMContract {
            deposit: TokenPairAmount {
                pair: token_pair.clone(),
                amount_0: Uint128::new(20000000000),
                amount_1: Uint128::new(20000000000),
            },
            expected_return: None,
            staking: Some(true),
            execute_sslp_virtual_swap: None,
        },
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_contract.address.to_string(),
            code_hash: amm_pair_contract.code_hash.to_string(),
        },
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        Some("20000000000uscrt"),
        &mut reports,
        None,
    )
    .unwrap();

    assert_eq!(
        get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(2000000000000 - 20000000000)
    );
    assert_eq!(
        get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
        Uint128::new(2000000000000 - 20000000000)
    );
    print_header("\n\tRegistering Tokens");
    handle(
        &RouterExecuteMsg::RegisterSNIP20Token {
            token_addr: usdt_token.address.clone(),
            token_code_hash: usdt_token.code_hash.clone(),
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
            token_addr: eth_token.address.clone(),
            token_code_hash: eth_token.code_hash.to_string(),
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

    print_header("\n\tRegistering Tokens against AMM PAIR");
    handle(
        &RouterExecuteMsg::RegisterSNIP20Token {
            token_addr: usdt_token.address.clone(),
            token_code_hash: usdt_token.code_hash.clone(),
        },
        &amm_pair_contract,
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
            token_addr: eth_token.address.clone(),
            token_code_hash: eth_token.code_hash.to_string(),
        },
        &amm_pair_contract,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )
    .unwrap();

    // GET PAIR INFO AND ASSERT LP TOKEN & STAKING
    let staking_contract = get_staking_contract(&amm_pair_contract.address.to_string()).unwrap();
    assert_ne!(staking_contract, None);

    print_header("GET PAIR INFO");
    let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
    let lp_token_info_query_unstake: AMMPairQueryMsgResponse = query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: amm_pair_contract.address.to_string(),
            code_hash: "".to_string(),
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
        fee_info: _,
    } = lp_token_info_query_unstake
    {
        println!(
            "\n\tLP Token Address {}",
            liquidity_token.address.to_string()
        );
        print_header("\n\tLP Token Liquidity - 20000000000");
        assert_eq!(total_liquidity, Uint128::new(20000000000));

        let get_stake_lp_token_info = StakingQueryMsg::WithPermit {
            permit: new_permit.clone(),
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

        if let StakingQueryMsgResponse::GetStakerLpTokenInfo {
            staked_lp_token,
            total_staked_lp_token,
        } = stake_lp_token_info
        {
            assert_eq!(staked_lp_token, Uint128::new(20000000000));
            assert_eq!(total_staked_lp_token, Uint128::new(20000000000));
        }

        print_header("\n\t 1. - BUY 100 ETH Initiating USDT to ETH Swap via AMM PAIR ");
        let old_usdt_balance = get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string());
        let old_eth_balance = get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string());
        handle(
            &snip20::ExecuteMsg::Send {
                recipient: amm_pair_contract.address.to_string(),
                amount: Uint128::new(10000u128),
                msg: Some(
                    to_binary(&AMMInvokeMsg::SwapTokens {
                        expected_return: Some(Uint128::new(10u128)),
                        to: Some(account.to_string()),
                        execute_arbitrage: None,
                    })
                    .unwrap(),
                ),
                padding: None,
                recipient_code_hash: None, //Some(pair_contract_code_hash.clone()),
                memo: None,
            },
            &usdt_token,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();

        assert_eq!(
            get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_eth_balance.u128() + 8901)
        );
        assert_eq!(
            get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_usdt_balance.u128() - 10000)
        );

        print_header("\n\t 1. - BUY 100 ETH Initiating USDT to ETH Swap via Router ");
        let old_usdt_balance = get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string());
        let old_eth_balance = get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string());
        handle(
            &snip20::ExecuteMsg::Send {
                recipient: router_contract.address.to_string(),
                amount: Uint128::new(10000u128),
                msg: Some(
                    to_binary(&RouterInvokeMsg::SwapTokensForExact {
                        expected_return: Some(Uint128::new(100)),
                        path: vec![Hop {
                            addr: amm_pair_contract.address.to_string(),
                            code_hash: pair_contract_code_hash.clone(),
                        }],
                        recipient: Some(account.to_string()),
                    })
                    .unwrap(),
                ),
                padding: None,
                recipient_code_hash: None,
                memo: None,
            },
            &usdt_token,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();

        assert_eq!(
            get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_eth_balance.u128() + 8901)
        );
        assert_eq!(
            get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_usdt_balance.u128() - 10000)
        );

        let old_usdt_balance = get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string());
        let old_eth_balance = get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string());
        print_header("\n\tAdding Liquidity to USDT/ETH - No Staking");
        handle(
            &AMMPairHandlMsg::AddLiquidityToAMMContract {
                deposit: TokenPairAmount {
                    pair: token_pair.clone(),
                    amount_0: Uint128::new(20000000000),
                    amount_1: Uint128::new(20000000000),
                },
                expected_return: None,
                staking: Some(false),
                execute_sslp_virtual_swap: None,
            },
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_contract.address.to_string(),
                code_hash: amm_pair_contract.code_hash.to_string(),
            },
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("20000000000uscrt"),
            &mut reports,
            None,
        )
        .unwrap();

        assert_eq!(
            get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_usdt_balance.u128() - 20000000000)
        );
        assert_eq!(
            get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_eth_balance.u128() - 20000000000)
        );

        let lp_token_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: liquidity_token.address.to_string(),
            code_hash: liquidity_token.code_hash.to_string(),
        };

        // SET VIEWING KEY
        set_viewing_key(
            VIEW_KEY,
            &lp_token_contract,
            &mut reports,
            ACCOUNT_KEY,
            "test",
        )
        .unwrap();
        assert_eq!(
            get_balance(
                &lp_token_contract,
                account.to_string(),
                VIEW_KEY.to_string()
            ),
            Uint128::new(19999980000)
        );

        let old_liquidity = (get_lp_liquidity(&amm_pair_contract.address.to_string())
            .unwrap()
            .unwrap());
        let old_usdt_balance = get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string());
        let old_eth_balance = get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string());
        print_header("\n\tSending Liquidity to USDT/ETH AMM Pair - Remove liquidity");
        handle(
            &snip20::ExecuteMsg::Send {
                recipient: amm_pair_contract.address.to_string(),
                amount: Uint128::new(9999996822),
                msg: Some(
                    to_binary(&AMMInvokeMsg::RemoveLiquidity {
                        from: None,
                        single_sided_withdraw_type: None, //None means 50/50 balanced withdraw, and a value here tells which token to send the withdraw in
                        single_sided_expected_return: None,
                    })
                    .unwrap(),
                ),
                padding: None,
                recipient_code_hash: None,
                memo: None,
            },
            &lp_token_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )
        .unwrap();
        assert_eq!(
            get_balance(
                &lp_token_contract,
                account.to_string(),
                VIEW_KEY.to_string()
            ),
            Uint128::new(9999983178)
        );

        // ASSERT LP TOKEN - 9999996822
        print_header("\n\tLP Token Liquidity - 9999996822");
        let new_liquidity =
            (get_lp_liquidity(&amm_pair_contract.address.to_string()).unwrap()).unwrap();
        assert_eq!(
            new_liquidity,
            Uint128::new(old_liquidity.u128() - 9999996822)
        );
        // ASSERT BALANCE USDT adn ETH TOKEN + 9999996822
        assert_eq!(
            get_balance(&usdt_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_usdt_balance.u128() + 10000006822)
        );
        assert_eq!(
            get_balance(&eth_token, account.to_string(), VIEW_KEY.to_string()),
            Uint128::new(old_eth_balance.u128() + 9999997371)
        );
        println!("\n\tUnstake 5000000000LP TOKEN");
    }

    return Ok(());
}

fn test_query_successful<Query: serde::Serialize, Response: serde::de::DeserializeOwned>(
    address: String,
    msg: Query,
) -> Result<Response> {
    query(
        &NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address,
            code_hash: "".to_string(),
        },
        msg,
        None,
    )
}
