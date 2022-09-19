// use cosmwasm_std::testing::mock_dependencies;
// use cosmwasm_std::testing::mock_env;
// use cosmwasm_std::to_binary;
// use cosmwasm_std::Addr;
// use cosmwasm_std::Deps;
// use cosmwasm_std::DepsMut;
// use cosmwasm_std::Env;
// use cosmwasm_std::{Api, Binary, CanonicalAddr, Querier, StdError, StdResult, Storage};
// use shadeswap_shared::amm_pair::AMMSettings;
// use shadeswap_shared::core::Fee;
// use shadeswap_shared::core::{ContractInstantiationInfo, ContractLink};
// use shadeswap_shared::msg::factory::InitMsg;
// pub use shadeswap_shared::{msg::factory::QueryResponse, Pagination};

// use crate::state::Config;

// #[cfg(test)]
// pub mod test_contract {
//     use super::*;
//     use crate::contract::execute;
//     use crate::contract::instantiate;
//     use crate::contract::query;
//     use crate::operations::create_pair;
//     use crate::state::config_r;
//     use crate::state::config_w;
//     use crate::state::PAGINATION_LIMIT;
//     use cosmwasm_std::from_binary;
//     use cosmwasm_std::Addr;
//     use cosmwasm_std::MessageInfo;
//     use shadeswap_shared::amm_pair::AMMPair;
//     use shadeswap_shared::core::TokenPair;
//     use shadeswap_shared::core::TokenType;
//     use shadeswap_shared::msg::factory::ExecuteMsg;
//     use shadeswap_shared::msg::factory::QueryMsg;
//     pub use shadeswap_shared::{msg::factory::QueryResponse, Pagination};

//     #[test]
//     fn init_ok() -> StdResult<()> {
//         let mut deps = mock_dependencies();
//         let config = mkconfig(0);
//         let env = mock_env();
//         assert!(instantiate(
//             deps.as_mut(),
//             env,
//             MessageInfo {
//                 sender: Addr::unchecked("admin"),
//                 funds: vec![]
//             },
//             (&config).into()
//         )
//         .is_ok());
//         assert_eq!(config, config_r(deps.as_ref().storage).load()?);
//         Ok(())
//     }

//     #[test]
//     fn get_set_config_ok() -> StdResult<()> {
//         let mut deps = mock_dependencies();
//         let env = mock_env();
//         instantiate(
//             deps.as_mut(),
//             env.clone(),
//             MessageInfo {
//                 sender: Addr::unchecked("admin"),
//                 funds: vec![],
//             },
//             (&mkconfig(0)).into(),
//         )?;

//         let new_config = mkconfig(5);
//         execute(
//             deps.as_mut(),
//             env,
//             MessageInfo {
//                 sender: Addr::unchecked("admin"),
//                 funds: vec![],
//             },
//             ExecuteMsg::SetConfig {
//                 pair_contract: Some(new_config.pair_contract.clone()),
//                 amm_settings: Some(new_config.amm_settings.clone()),
//                 lp_token_contract: Some(new_config.lp_token_contract.clone()),
//             },
//         )
//         .unwrap();

//         let response: QueryResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})?)?;
//         let compare: QueryResponse = (&new_config).into();
//         assert_eq!(compare, response);
//         Ok(())
//     }

//     // #[test]
//     // fn register_amm_pair_ok() -> StdResult<()> {
//     //     let ref mut deps = mock_dependencies();
//     //     let env = mock_env();
//     //     let config = mkconfig(0);

//     //     config_w(deps.as_mut().storage).save(&config)?;

//     //     let signature = create_signature(&env)?;
//     //     save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

//     //     let pair = TokenPair(
//     //         TokenType::CustomToken {
//     //             contract_addr: Addr::unchecked("token_addr".into()),
//     //             token_code_hash: "13123adasd".into(),
//     //         },
//     //         TokenType::NativeToken {
//     //             denom: "test1".into(),
//     //         },
//     //     );

//     //     execute(
//     //         deps,
//     //         env,
//     //         MessageInfo {
//     //             sender: Addr::unchecked("admin"),
//     //             funds: vec![]
//     //         },
//     //         ExecuteMsg::RegisterAMMPair {
//     //             pair: pair.clone(),
//     //             signature,
//     //         },
//     //     )?;

//     //     let result: Option<Binary> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
//     //     match result {
//     //         None => {}
//     //         _ => panic!("Ephemeral storage should be empty!"),
//     //     }

//     //     Ok(())
//     // }

//     #[test]
//     fn create_pair_ok() -> StdResult<()> {
//         let ref mut deps = mock_dependencies();
//         let env = mock_env();
//         let config = mkconfig(0);
//         assert!(instantiate(
//             deps.as_mut(),
//             env,
//             MessageInfo {
//                 sender: Addr::unchecked("admin"),
//                 funds: vec![]
//             },
//             (&config).into()
//         )
//         .is_ok());

//         let pair = TokenPair(
//             TokenType::CustomToken {
//                 contract_addr: Addr::unchecked("token_addr"),
//                 token_code_hash: "diff".into(),
//             },
//             TokenType::CustomToken {
//                 contract_addr: Addr::unchecked("token_addr"),
//                 token_code_hash: "13123adasd".into(),
//             },
//         );

//         let result = create_pair(deps.as_mut(), mock_env(), pair,  Addr::unchecked("admin"),to_binary(&"entropy").unwrap(), None);
//         assert!(result.is_ok());
//         Ok(())
//     }
//     #[test]
//     fn add_amm_pairs() {
//         let ref mut deps = mock_dependencies();
//         let config = mkconfig(0);
//         let env = mock_env();

//         instantiate(deps.as_mut(), env.clone(), MessageInfo {
//             sender: Addr::unchecked("admin"),
//             funds: vec![]
//         },(&config).into()).unwrap();

//         let mut amm_pairs: Vec<AMMPair> = vec![];

//         for i in 0..5 {
//             amm_pairs.push(AMMPair {
//                 pair: TokenPair(
//                     TokenType::CustomToken {
//                         contract_addr: Addr::unchecked(format!("token_0_addr_{}", i)),
//                         token_code_hash: format!("token_0_hash_{}", i),
//                     },
//                     TokenType::CustomToken {
//                         contract_addr: Addr::unchecked(format!("token_1_addr_{}", i)),
//                         token_code_hash: format!("token_1_hash_{}", i),
//                     },
//                 ),
//                 address: Addr::unchecked(format!("pair_addr_{}", i)),
//                 enabled: true,
//             });
//         }

//         execute(
//             deps.as_mut(),
//             env,
//             MessageInfo {
//                 sender: Addr::unchecked("admin"),
//                 funds: vec![]
//             },
//             ExecuteMsg::AddAMMPairs {
//                 amm_pairs: amm_pairs.clone()[0..].into(),
//             },
//         )
//         .unwrap();

//         let result = query(
//             deps.as_ref(),
//             mock_env(),
//             QueryMsg::ListAMMPairs {
//                 pagination: pagination(0, PAGINATION_LIMIT),
//             },
//         )
//         .unwrap();

//         let response: QueryResponse = from_binary(&result).unwrap();

//         match response {
//             QueryResponse::ListAMMPairs { amm_pairs: stored } => {
//                 assert_eq!(amm_pairs, stored)
//             }
//             _ => panic!("QueryResponse::ListExchanges"),
//         }
//     }

//     /*
//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // anyone can increment
//         let env = mock_env("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(&mut deps, env, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // not anyone can reset
//         let unauth_env = mock_env("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(&mut deps, unauth_env, msg);
//         match res {
//             Err(StdError::Unauthorized { .. }) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_env = mock_env("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(&mut deps, auth_env, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }*/
// }

// // pub mod test_state {
// //     use shadeswap_shared::{amm_pair::AMMPair, core::Canonize};

// //     use super::*;

// //     fn swap_pair<A: Clone>(pair: &TokenPair) -> TokenPair {
// //         TokenPair(pair.1.clone(), pair.0.clone())
// //     }

// //     #[test]
// //     fn generate_pair_key_ok() -> StdResult<()> {
// //         fn cmp_pair<S: Storage, A: Api, Q: Querier>(
// //             deps: &Deps<S, A, Q>,
// //             pair: TokenPair<HumanAddr>,
// //         ) -> StdResult<()> {
// //             let stored_pair = pair.clone().canonize(&deps.api)?;
// //             let key = generate_pair_key(&stored_pair);

// //             let pair = swap_pair(&pair.clone());

// //             let stored_pair = pair.canonize(&deps.api)?;
// //             let swapped_key = generate_pair_key(&stored_pair);

// //             assert_eq!(key, swapped_key);

// //             Ok(())
// //         }

// //         let ref deps = mock_dependencies();

// //         cmp_pair(
// //             deps,
// //             TokenPair(
// //                 TokenType::CustomToken {
// //                     contract_addr: Addr::unchecked("first_addr".into()),
// //                     token_code_hash: "13123adasd".into(),
// //                 },
// //                 TokenType::CustomToken {
// //                     contract_addr: Addr::unchecked("scnd_addr".into()),
// //                     token_code_hash: "4534qwerqqw".into(),
// //                 },
// //             ),
// //         )?;

// //         cmp_pair(
// //             deps,
// //             TokenPair(
// //                 TokenType::NativeToken {
// //                     denom: "test1".into(),
// //                 },
// //                 TokenType::NativeToken {
// //                     denom: "test2".into(),
// //                 },
// //             ),
// //         )?;

// //         cmp_pair(
// //             deps,
// //             TokenPair(
// //                 TokenType::NativeToken {
// //                     denom: "test3".into(),
// //                 },
// //                 TokenType::CustomToken {
// //                     contract_addr: Addr::unchecked("third_addr".into()),
// //                     token_code_hash: "asd21312asd".into(),
// //                 },
// //             ),
// //         )?;

// //         Ok(())
// //     }

// //     #[test]
// //     fn store_and_get_amm_pairs_ok() {
// //         let ref mut deps = mock_dependencies();
// //         let mut amm_pairs: Vec<AMMPair> = vec![];
// //         amm_pairs.push(AMMPair {
// //             pair: TokenPair(
// //                 TokenType::CustomToken {
// //                     contract_addr: format!("token_0_addr_{}", 0).into(),
// //                     token_code_hash: format!("token_0_hash_{}", 0),
// //                 },
// //                 TokenType::CustomToken {
// //                     contract_addr: format!("token_1_addr_{}", 0).into(),
// //                     token_code_hash: format!("token_1_hash_{}", 0),
// //                 },
// //             ),
// //             address: format!("pair_addr_{}", 0).into(),
// //         });
// //         save_amm_pairs(deps, amm_pairs.clone()).unwrap();
// //         let result = load_amm_pairs(deps, pagination(0, 1)).unwrap();

// //         //Check Count was updated
// //         assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());

// //         //Check number of result was returned
// //         assert_eq!(1, result.len());

// //         //Match result
// //         assert_eq!(amm_pairs[0], result[0]);
// //     }

// //     #[test]
// //     fn save_and_load_amm_pairs_count_ok() {
// //         let ref mut deps = mock_dependencies();
// //         save_amm_pairs_count(&mut deps.storage, 1).unwrap();
// //         assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());
// //         assert_ne!(2, load_amm_pairs_count(&mut deps.storage).unwrap())
// //     }
// // }

// fn mkconfig(id: u64) -> Config {
//     Config::from_init_msg(InitMsg {
//         pair_contract: ContractInstantiationInfo {
//             id,
//             code_hash: "2341586789".into(),
//         },
//         amm_settings: AMMSettings {
//             lp_fee: Fee::new(28, 10000),
//             shade_dao_fee: Fee::new(2, 10000),
//             shade_dao_address: ContractLink {
//                 address: Addr::unchecked("CALLBACKADDR"),
//                 code_hash: "Test".to_string(),
//             },
//         },
//         lp_token_contract: ContractInstantiationInfo {
//             id,
//             code_hash: "123".into(),
//         },
//         prng_seed: to_binary(&"prng").unwrap(),
//     })
// }

// fn pagination(start: u64, limit: u8) -> Pagination {
//     Pagination { start, limit }
// }

// impl Into<InitMsg> for &Config {
//     fn into(self) -> InitMsg {
//         InitMsg {
//             pair_contract: self.pair_contract.clone(),
//             amm_settings: AMMSettings {
//                 lp_fee: Fee::new(28, 10000),
//                 shade_dao_fee: Fee::new(2, 10000),
//                 shade_dao_address: ContractLink {
//                     address: Addr::unchecked("CALLBACKADDR"),
//                     code_hash: "Test".to_string(),
//                 },
//             },
//             lp_token_contract: self.lp_token_contract.clone(),
//             prng_seed: to_binary(&"prng").unwrap(),
//         }
//     }
// }

// impl Into<QueryResponse> for &Config {
//     fn into(self) -> QueryResponse {
//         QueryResponse::GetConfig {
//             pair_contract: self.pair_contract.clone(),
//             amm_settings: self.amm_settings.clone(),
//             lp_token_contract: self.lp_token_contract.clone(),
//         }
//     }
// }
