use shadeswap_shared::viewing_keys::ViewingKey;
use shadeswap_shared::custom_fee::Fee;
use cosmwasm_std::Uint128;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::to_binary;
use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY, SNIP20_FILE,
    STAKING_FILE, VIEW_KEY,
};
use cosmwasm_std::BalanceResponse;
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};

use serde_json::Result;
use shadeswap_shared::core::ContractInstantiationInfo;
use shadeswap_shared::secret_toolkit::snip20::ExecuteMsg;
use shadeswap_shared::secret_toolkit::snip20::QueryMsg;
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings},
    msg::{
        amm_pair::{
            ExecuteMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg,
            QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryMsgResponse,
        },
        factory::{
            ExecuteMsg as FactoryExecuteMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        router::{
            ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg,
        },
    },
    stake_contract::StakingContractInit,
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType, core::ContractLink,
};
use std::env;

use snip20_reference_impl::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

pub const ACCOUNT_KEY: &str = "deployer";
pub const STORE_GAS: &str = "10000000";

pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: view_key,
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.amount.amount
}

fn main() -> serde_json::Result<()> {
    // let mut reports = vec![];
    // let s_sCRT = NetContract {
    //     label: "iAKBCfPG".to_string(),
    //     id: "10367".to_string(),
    //     address: "secret15nc5yhefuwun9wxghzmhq0fzrswcyksz9zqvs5".to_string(),
    //     code_hash: "A3AB7A262D42D9FD4F67ABD507DB7E1237A68AE9EF57B746AA74AD52F922583B".to_string(),
    // };

    // let s_sSHD = NetContract {
    //     label: "iAKBCfPG".to_string(),
    //     id: "10368".to_string(),
    //     address: "secret1dcxn4nqexw5d6gq2fksvnevynxaeu7nmlu4ejl".to_string(),
    //     code_hash: "A3AB7A262D42D9FD4F67ABD507DB7E1237A68AE9EF57B746AA74AD52F922583B".to_string(),
    // };

    // // let s_sREWARDSNIP20 = NetContract {
    // //     label: "TNZElupy".to_string(),
    // //     id: "10369".to_string(),
    // //     address: "secret156jue4d0qpfnl6klpw7xgz6h3dv52lcthnpl82".to_string(),
    // //     code_hash: "A3AB7A262D42D9FD4F67ABD507DB7E1237A68AE9EF57B746AA74AD52F922583B".to_string(),
    // // };

    // let test_pair = TokenPair(
    //     TokenType::CustomToken {
    //         contract_addr: s_sCRT.address.clone().into(),
    //         token_code_hash: s_sCRT.code_hash.to_string(),
    //     },
    //     TokenType::CustomToken {
    //         contract_addr: s_sSHD.address.clone().into(),
    //         token_code_hash: s_sSHD.code_hash.to_string(),
    //     },
    // );

    let msg = FactoryHandleMsg::SetConfig {
        pair_contract: None,
        lp_token_contract: None,
        amm_settings: Some( AMMSettings {
            lp_fee: Fee::new(8, 100),
            shade_dao_fee: Fee::new(2, 100),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from("secret1hfvezhepf6ahwry0gzhcra6zsdmva5xhphhzdh".to_string())),
                code_hash: "".to_string(),
            },
        }),
    };

    // handle(
    //     &RouterExecuteMsg::RegisterSNIP20Token {
    //         token: HumanAddr::from(s_sCRT.address.clone()),
    //         token_code_hash: s_sCRT.code_hash.to_string(),
    //     },
    //     &NetContract { label:  "".to_string(), id:  "".to_string(), address:"secret1qp4scfkayaust4uxashax6t36upx8q32auh263".to_string(), code_hash: "".to_string() },
    //     ACCOUNT_KEY,
    //     Some(GAS),
    //     Some("test"),
    //     None,
    //     &mut reports,
    //     None,
    // )
    // .unwrap();

    // handle(
    //     &RouterExecuteMsg::RegisterSNIP20Token {
    //         token: HumanAddr::from(s_sSHD.address.clone()),
    //         token_code_hash: s_sSHD.code_hash.to_string(),
    //     },
    //     &NetContract { label:  "".to_string(), id:  "".to_string(), address:"secret1qp4scfkayaust4uxashax6t36upx8q32auh263".to_string(), code_hash: "".to_string() },
    //     ACCOUNT_KEY,
    //     Some(GAS),
    //     Some("test"),
    //     None,
    //     &mut reports,
    //     None,
    // )
    // .unwrap();



    // {
    //     let msg = snip20_reference_impl::msg::ExecuteMsg::SetViewingKey {
    //         key: String::from(VIEW_KEY),
    //         padding: None,
    //     };
    //     handle(
    //         &msg,
    //         &s_sSHD,
    //         ACCOUNT_KEY,
    //         Some(GAS),
    //         Some("test"),
    //         None,
    //         &mut reports,
    //         None,
    //     )?;
    // }

    // println!(
    //     "{}",
    //     get_balance(&s_sCRT, ACCOUNT_KEY.to_string(), VIEW_KEY.to_string(),)
    // );
    
    handle(
        &HandleMsg::Send {
            recipient: HumanAddr::from("secret18letgdtj6fz55u4a9fm5hal9tez3ruz79gscpj".to_string()),
            recipient_code_hash: None,
            amount: Uint128(25000),
            msg: Some(
                to_binary(&RouterInvokeMsg::SwapTokensForExact {
                    expected_return: Some(Uint128(5)),
                    paths: vec![HumanAddr::from("secret1c50a69q0dxcedu3ufs40sf7rupz6grmxnhgwtn".to_string())],
                    recipient: None,
                })
                .unwrap(),
            ),
            memo: None,
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




    // println!("\n\tDepositing 1000000000uscrt sSCRT");

    // {
    //     let msg = snip20::HandleMsg::Mint { padding: None, recipient: HumanAddr(String::from("secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q")), amount: Uint128(1000000000)};

    //     handle(
    //         &msg,
    //         &s_sSHD,
    //         ACCOUNT_KEY,
    //         Some(GAS),
    //         Some("test"),
    //         Some("10000000uscrt"),
    //         &mut reports,
    //         None,
    //     )?;
    // }

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
            id: "".to_string(),
            address: "secret15hmyq33a4rn8d82h8gtmd3nyxq04zdc89u32p7".to_string(),
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

    println!("\n\tDepositing 1000000000uscrt sSHD");

    // {
    //     let msg = snip20::ExecuteMsg::Deposit { padding: None };

    //     handle(
    //         &msg,
    //         &s_sSHD,
    //         ACCOUNT_KEY,
    //         Some(GAS),
    //         Some("test"),
    //         Some("1000000000uscrt"),
    //         &mut reports,
    //         None,
    //     )?;
    // }

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
                address: HumanAddr(String::from("secret1hfvezhepf6ahwry0gzhcra6zsdmva5xhphhzdh".to_string())),
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
                    address: HumanAddr(String::from(factory_contract.address.clone())),
                    code_hash: factory_contract.code_hash.clone(),
                },
                entropy: to_binary(&"".to_string()).unwrap(),
                viewing_key: Some(VIEW_KEY.to_string()),
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
                &RouterHandleMsg::RegisterSNIP20Token {
                    token: HumanAddr::from(s_sCRT.address.clone()),
                    token_code_hash: s_sCRT.code_hash.to_string(),
                },
                &NetContract { label:  "".to_string(), id:  "".to_string(), address: router_contract.address.to_string(), code_hash: "".to_string() },
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
                &NetContract { label:  "".to_string(), id:  "".to_string(), address: router_contract.address.to_string().to_string(), code_hash: "".to_string() },
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
                    &FactoryHandleMsg::CreateAMMPair {
                        pair: test_pair.clone(),
                        entropy: entropy,
                        staking_contract: Some(StakingContractInit {
                            contract_info: ContractInstantiationInfo {
                                code_hash: staking_contract.code_hash.to_string(),
                                id: staking_contract.id.clone().parse::<u64>().unwrap(),
                            },
                            amount: Uint128(100000u128),
                            reward_token: TokenType::CustomToken {
                                contract_addr: s_sCRT.address.clone().into(),
                                token_code_hash: s_sCRT.code_hash.to_string(),
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
                        &snip20_reference_impl::msg::HandleMsg::IncreaseAllowance {
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
                        &snip20_reference_impl::msg::HandleMsg::IncreaseAllowance {
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
    // TODO NEED TO ADD MINT
    println!("\n\tDepositing 1000000000uscrt sSCRT");

    {
        let msg = snip20_reference_impl::msg::HandleMsg::Mint { padding: None, recipient: HumanAddr(String::from("secret1ss0c4wgzcuszfsnaf0r32gpwkx2ssldqtz4mf5")), amount: Uint128(10000000000), memo: None };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("10000000uscrt"),
            &mut reports,
            None,
        )?;
    }
                }
            }
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}

//Used for adapting
// let test_pair = TokenPair(
//     TokenType::CustomToken {
//         contract_addr: "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg".to_string().into(),
//         token_code_hash: "9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813".to_string(),
//     },
//     TokenType::CustomToken {
//         contract_addr: "secret19ymc8uq799zf36wjsgu4t0pk8euddxtx5fggn8".to_string().into(),
//         token_code_hash: "5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1".to_string(),
//     },
// );

// handle(
//     &RouterExecuteMsg::RegisterSNIP20Token {
//         token: HumanAddr::from("secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg".to_string()),
//         token_code_hash: "9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813".to_string(),
//     },
//     &NetContract { label:  "".to_string(), id:  "".to_string(), address: "secret1fxnrc2qda4c7p2qsu6k0yuu9rsaqdez54zr0q3".to_string(), code_hash: "".to_string() },
//     ACCOUNT_KEY,
//     Some(GAS),
//     Some("test"),
//     None,
//     &mut reports,
//     None,
// )
// .unwrap();

// handle(
//     &RouterExecuteMsg::RegisterSNIP20Token {
//         token: HumanAddr::from("secret19ymc8uq799zf36wjsgu4t0pk8euddxtx5fggn8".to_string()),
//         token_code_hash: "5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1".to_string(),
//     },
//     &NetContract { label:  "".to_string(), id:  "".to_string(), address: "secret1fxnrc2qda4c7p2qsu6k0yuu9rsaqdez54zr0q3".to_string(), code_hash: "".to_string() },
//     ACCOUNT_KEY,
//     Some(GAS),
//     Some("test"),
//     None,
//     &mut reports,
//     None,
// )
// .unwrap();


// {
//     handle(
//         &FactoryExecuteMsg::CreateAMMPair {
//             pair: test_pair.clone(),
//             entropy:  to_binary(&"".to_string()).unwrap(),
//             staking_contract: None,
//             // staking_contract: Some(StakingContractInit {
//             //     contract_info: ContractInstantiationInfo {
//             //         code_hash: staking_contract.code_hash.to_string(),
//             //         id: staking_contract.id.clone().parse::<u64>().unwrap(),
//             //     },
//             //     amount: Uint128(100000u128),
//             //     reward_token: TokenType::CustomToken {
//             //         contract_addr: s_sCRT.address.clone().into(),
//             //         token_code_hash: s_sCRT.code_hash.to_string(),
//             //     },
//             // }),
//         },
//         &NetContract { label: "I7nZ28Aq".to_string(), id: "11425".to_string(), address: "secret1fxnrc2qda4c7p2qsu6k0yuu9rsaqdez54zr0q3".to_string(), code_hash: "71EB188450FBE579E5601AF81D8416890E502BA3A2799B693185B304239F2E20".to_string() },
//         ACCOUNT_KEY,
//         Some(GAS),
//         Some("test"),
//         None,
//         &mut reports,
//         None,
//     )
//     .unwrap();
// }