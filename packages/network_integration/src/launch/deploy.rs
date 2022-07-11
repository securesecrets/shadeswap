use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY, SNIP20_FILE,
    STAKING_FILE, VIEW_KEY,
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
        Callback, ContractInstantiationInfo, ContractLink, ViewingKey,
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
    },
    stake_contract::StakingContractInit,
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType,
};
use std::env;

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

pub const ACCOUNT_KEY: &str = "deployer";
pub const STORE_GAS: &str = "10000000";

fn main() -> serde_json::Result<()> {
    print_header("Storing all contracts");
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];
    print_warning("Storing LP Token Contract");
    let s_lp = store_and_return_contract(
        &LPTOKEN20_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;
    print_warning("Storing AMM Pair Token Contract");
    let s_ammPair = store_and_return_contract(
        &AMM_PAIR_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;
    print_warning("Storing Staking Contract");
    let staking_contract = store_and_return_contract(
        &STAKING_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;

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
        &FACTORY_FILE.replace("../", ""),
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
                &ROUTER_FILE.replace("../", ""),
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
                None,
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
                Some(&SNIP20_FILE.replace("../", "")),
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
                    assert_eq!(amm_pairs.len(), 2);
                    let ammPair = amm_pairs[0].clone();
                    let amm_pair_2 = amm_pairs[1].clone();

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

                    handle(
                        &snip20::HandleMsg::IncreaseAllowance {
                            spender: HumanAddr(String::from(amm_pair_2.address.0.to_string())),
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
                            staking: None
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
