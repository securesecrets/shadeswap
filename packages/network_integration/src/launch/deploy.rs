use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::BalanceResponse;
use cosmwasm_std::Uint128;
use network_integration::utils::InitConfig;
use network_integration::utils::ADMIN_FILE;
use network_integration::utils::API_KEY;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_warning, AMM_PAIR_FILE,
    FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SNIP20_FILE, STAKING_FILE, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{handle, init, query, store_and_return_contract},
};

use shadeswap_shared::core::ContractInstantiationInfo;
use shadeswap_shared::core::Fee;
use shadeswap_shared::core::TokenPair;
use shadeswap_shared::core::TokenPairAmount;
use shadeswap_shared::core::TokenType;
use shadeswap_shared::snip20::QueryMsg;
use shadeswap_shared::Contract;
use shadeswap_shared::{
    amm_pair::AMMSettings,
    contract_interfaces::admin::InstantiateMsg as AdminInstantiateMsg,
    msg::{
        amm_pair::ExecuteMsg as AMMPairHandlMsg,
        factory::{
            ExecuteMsg as FactoryExecuteMsg, InitMsg as FactoryInitMsg,
            QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse,
        },
        router::{ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg},
        staking::StakingContractInit,
    },
    Pagination,
};

use shadeswap_shared::snip20 as snip20_reference_impl;

pub const ACCOUNT_KEY: &str = "deployer";
pub const STORE_GAS: &str = "10000000";

pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = QueryMsg::Balance {
        address: from,
        key: view_key,
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.amount.amount
}

fn main() -> serde_json::Result<()> {
    //redeploy_infra()?;
    deploy_fresh()?;
    return Ok(());
}

#[allow(dead_code)]
fn deploy_fresh() -> serde_json::Result<()> {
    let mut reports = vec![];
    print_warning("SENT");
    print_warning("Storing LP Token Contract");
    let (_btc_init, btc_contract) = init_snip20(
        "Bitcoin".to_string(),
        "BTC".to_string(),
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
        Some(&SNIP20_FILE.replace("../", "")),
    )?;
    print_contract(&btc_contract);

    let (_usdt_init, usdt_contract) = init_snip20(
        "USDT".to_string(),
        "USDT".to_string(),
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
        Some(&SNIP20_FILE.replace("../", "")),
    )?;
    print_contract(&usdt_contract);

    let (_eth_init, eth_contract) = init_snip20(
        "ETH".to_string(),
        "ETH".to_string(),
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
        Some(&SNIP20_FILE.replace("../", "")),
    )?;
    print_contract(&eth_contract);

    {
        let msg = snip20_reference_impl::ExecuteMsg::Deposit { padding: None };

        handle(
            &msg,
            &eth_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;

        let msg = snip20_reference_impl::ExecuteMsg::Deposit { padding: None };

        handle(
            &msg,
            &usdt_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;

        let msg = snip20_reference_impl::ExecuteMsg::Deposit { padding: None };

        handle(
            &msg,
            &btc_contract,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000000uscrt"),
            &mut reports,
            None,
        )?;

        {
            let msg = snip20_reference_impl::ExecuteMsg::SetViewingKey {
                key: String::from(VIEW_KEY),
                padding: None,
            };
            handle(
                &msg,
                &btc_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )?;
        }

        {
            let msg = snip20_reference_impl::ExecuteMsg::SetViewingKey {
                key: String::from(VIEW_KEY),
                padding: None,
            };
            handle(
                &msg,
                &usdt_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )?;
        }

        {
            let msg = snip20_reference_impl::ExecuteMsg::SetViewingKey {
                key: String::from(VIEW_KEY),
                padding: None,
            };
            handle(
                &msg,
                &eth_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )?;
        }

        {
            let msg = snip20_reference_impl::ExecuteMsg::Mint {
                padding: None,
                recipient: "secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q".to_string(),
                amount: Uint128::new(100000000000000u128),
                memo: None,
            };

            handle(
                &msg,
                &usdt_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("10000000uscrt"),
                &mut reports,
                None,
            )?;
        }
        {
            let msg = snip20_reference_impl::ExecuteMsg::Mint {
                padding: None,
                recipient: "secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q".to_string(),
                amount: Uint128::new(100000000000000000000000u128),
                memo: None,
            };

            handle(
                &msg,
                &eth_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("10000000uscrt"),
                &mut reports,
                None,
            )?;
        }
        {
            let msg = snip20_reference_impl::ExecuteMsg::Mint {
                padding: None,
                recipient: "secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q".to_string(),
                amount: Uint128::new(100000000000000000u128),
                memo: None,
            };

            handle(
                &msg,
                &btc_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                Some("10000000uscrt"),
                &mut reports,
                None,
            )?;
        }
    }

    let test_pair = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(usdt_contract.address.clone()),
            token_code_hash: usdt_contract.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(btc_contract.address.clone()),
            token_code_hash: btc_contract.code_hash.to_string(),
        },
    );

    let test_pair_2 = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(usdt_contract.address.clone()),
            token_code_hash: usdt_contract.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(eth_contract.address.clone()),
            token_code_hash: eth_contract.code_hash.to_string(),
        },
    );

    print_header("Storing all contracts");
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    print_warning("Storing LP Token Contract");
    let s_lp = store_and_return_contract(
        &LPTOKEN20_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;

    print_warning("Storing AMM Pair Token Contract");
    let s_amm_pair = store_and_return_contract(
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

    let _admin_contract = store_and_return_contract(
        &ADMIN_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;

    print_header("\n\tInitializing Admin Contract");

    let admin_msg = AdminInstantiateMsg {
        super_admin: Some("secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q".to_string()),
    };

    let admin_contract = init(
        &admin_msg,
        &ADMIN_FILE.replace("../", ""),
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    print_contract(&admin_contract);

    print_header("\n\tInitializing Factory Contract");

    let factory_msg = FactoryInitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: s_amm_pair.code_hash.to_string(),
            id: s_amm_pair.id.clone().parse::<u64>().unwrap(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(8, 100),
            shade_dao_fee: Fee::new(2, 100),
            shade_dao_address: Contract {
                address: Addr::unchecked(
                    "secret1hfvezhepf6ahwry0gzhcra6zsdmva5xhphhzdh".to_string(),
                ),
                code_hash: "".to_string(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
        api_key: API_KEY.to_string(),
        authenticator: None,
        admin_auth: Contract {
            address: Addr::unchecked(admin_contract.address.to_string()),
            code_hash: admin_contract.code_hash.clone(),
        },
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
                entropy: to_binary(&"".to_string()).unwrap(),
                admin_auth: Contract {
                    address: Addr::unchecked(admin_contract.address.to_string()),
                    code_hash: admin_contract.code_hash.clone(),
                },
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

            handle(
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: usdt_contract.address.clone(),
                    token_code_hash: usdt_contract.code_hash.to_string(),
                },
                &NetContract {
                    label: "".to_string(),
                    id: "".to_string(),
                    address: router_contract.address.to_string(),
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
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: btc_contract.address.clone(),
                    token_code_hash: btc_contract.code_hash.to_string(),
                },
                &NetContract {
                    label: "".to_string(),
                    id: "".to_string(),
                    address: router_contract.address.to_string().to_string(),
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
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: eth_contract.address.clone(),
                    token_code_hash: eth_contract.code_hash.to_string(),
                },
                &NetContract {
                    label: "".to_string(),
                    id: "".to_string(),
                    address: router_contract.address.to_string().to_string(),
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

            {
                handle(
                    &FactoryExecuteMsg::CreateAMMPair {
                        pair: test_pair.clone(),
                        entropy: entropy.clone(),
                        staking_contract: Some(StakingContractInit {
                            contract_info: ContractInstantiationInfo {
                                code_hash: staking_contract.code_hash.to_string(),
                                id: staking_contract.id.clone().parse::<u64>().unwrap(),
                            },
                            daily_reward_amount: Uint128::from(100000u128),
                            reward_token: TokenType::CustomToken {
                                contract_addr: Addr::unchecked(usdt_contract.address.clone()),
                                token_code_hash: usdt_contract.code_hash.to_string(),
                            },
                            valid_to: Uint128::new(3747905010000u128),
                            decimals: 18u8,
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

            {
                handle(
                    &FactoryExecuteMsg::CreateAMMPair {
                        pair: test_pair_2.clone(),
                        entropy: entropy,
                        staking_contract: Some(StakingContractInit {
                            contract_info: ContractInstantiationInfo {
                                code_hash: staking_contract.code_hash.to_string(),
                                id: staking_contract.id.clone().parse::<u64>().unwrap(),
                            },
                            daily_reward_amount: Uint128::from(100000u128),
                            reward_token: TokenType::CustomToken {
                                contract_addr: Addr::unchecked(usdt_contract.address.clone()),
                                token_code_hash: usdt_contract.code_hash.to_string(),
                            },
                            valid_to: Uint128::new(3747905010000u128),
                            decimals: 18u8,
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
                    {
                        let amm_pair = amm_pairs[0].clone();

                        print_header("\n\tAdding Liquidity to Pair Contract");
                        handle(
                            &snip20_reference_impl::ExecuteMsg::IncreaseAllowance {
                                spender: amm_pair.address.to_string(),
                                amount: Uint128::from(1000000000u64),
                                expiration: None,
                                padding: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: btc_contract.id.clone(),
                                address: btc_contract.address.clone(),
                                code_hash: btc_contract.code_hash.to_string(),
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
                            &snip20_reference_impl::ExecuteMsg::IncreaseAllowance {
                                spender: amm_pair.address.to_string(),
                                amount: Uint128::from(200000000000u64),
                                expiration: None,
                                padding: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: usdt_contract.id.clone(),
                                address: usdt_contract.address.clone(),
                                code_hash: usdt_contract.code_hash.to_string(),
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
                                    pair: test_pair.clone(),
                                    amount_0: Uint128::from(200000000000u64),
                                    amount_1: Uint128::from(1000000000u64),
                                },
                                expected_return: None,

                                staking: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: s_amm_pair.id.clone(),
                                address: amm_pair.address.to_string(),
                                code_hash: s_amm_pair.code_hash.to_string(),
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

                    {
                        let amm_pair = amm_pairs[1].clone();

                        print_header("\n\tAdding Liquidity to Pair Contract");
                        handle(
                            &snip20_reference_impl::ExecuteMsg::IncreaseAllowance {
                                spender: amm_pair.address.to_string(),
                                amount: Uint128::from(1308000000000000000000000u128),
                                expiration: None,
                                padding: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: eth_contract.id.clone(),
                                address: eth_contract.address.clone(),
                                code_hash: eth_contract.code_hash.to_string(),
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
                            &snip20_reference_impl::ExecuteMsg::IncreaseAllowance {
                                spender: amm_pair.address.to_string(),
                                amount: Uint128::from(200000000000u64),
                                expiration: None,
                                padding: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: usdt_contract.id.clone(),
                                address: usdt_contract.address.clone(),
                                code_hash: usdt_contract.code_hash.to_string(),
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
                                    pair: test_pair_2.clone(),
                                    amount_0: Uint128::from(13_080_000_000u64),
                                    amount_1: Uint128::from(1000000000000000000u128),
                                },
                                expected_return: None,
                                staking: None,
                            },
                            &NetContract {
                                label: "".to_string(),
                                id: s_amm_pair.id.clone(),
                                address: amm_pair.address.to_string(),
                                code_hash: s_amm_pair.code_hash.to_string(),
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
            }
        } else {
            assert!(false, "Query returned unexpected response");
        }
    }
    return Ok(());
}
