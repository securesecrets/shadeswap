// use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, ExecuteMsg,SwapInfo, SwapResult,  InvokeMsg, QueryMsgResponse}};
// use shadeswap_shared::token_amount::{{TokenAmount}};
// use shadeswap_shared::token_pair::{{TokenPair}};
// use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
// use shadeswap_shared::amm_pair::{{AMMPair, AMMSettings}};
// use secret_toolkit::snip20::TokenInfo;
// use shadeswap_shared::msg::amm_pair::{{ TradeHistory}};
// use cosmwasm_std::testing::mock_dependencies;
// use crate::state::amm_pair_storage::{{ store_config, load_config,
//     remove_whitelist_address,is_address_in_whitelist, add_whitelist_address,load_whitelist_address, }};
// use crate::contract::init;
// use crate::contract::{{create_viewing_key, calculate_price, calculate_swap_result,swap, query, add_liquidity, handle}};
// use std::hash::Hash;
// use cosmwasm_std::{BankQuery, AllBalanceResponse, to_vec,log,  Coin, StdResult, HumanAddr, BalanceResponse, from_binary, StdError, QueryRequest, Empty, Uint128, to_binary, QuerierResult, from_slice, Querier, testing::{MockApi, MockStorage}, Extern, ContractInfo, MessageInfo, BlockInfo, Env, Api, Storage, WasmQuery};
// use secret_toolkit::snip20::Balance;
// use shadeswap_shared::{core::ContractLink};
// use crate::state::{{Config}};    
// use shadeswap_shared::token_type::TokenType;
// use serde::Deserialize;
// use serde::Serialize;
// use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
// pub const FACTORY_CONTRACT_ADDRESS: &str = "FACTORY_CONTRACT_ADDRESS";
// pub const CUSTOM_TOKEN_1: &str = "CUSTOM_TOKEN_1";
// pub const CUSTOM_TOKEN_2: &str = "CUSTOM_TOKEN_2";
// pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";
// pub const LP_TOKEN_ADDRESS: &str = "LP_TOKEN_ADDRESS";
// use crate::help_math::calculate_and_print_price;

// #[cfg(test)]
// pub mod tests {
//     use super::*;   
//     use serde::de::DeserializeOwned;
//     use shadeswap_shared::custom_fee::Fee;
//     use shadeswap_shared::core::{Callback, ContractInstantiationInfo};
//     use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };   
//     use crate::state::amm_pair_storage::{{store_trade_history, load_trade_history, load_trade_counter}};
//     use crate::state::tradehistory::{{ DirectionType}};
//     use crate::test::help_test_lib::mkenv;  
//     use serde::Deserialize;
//     use serde::Serialize;

//     #[test]
//     fn assert_init_config() -> StdResult<()> {       
//         // let info = mock_info("amm_pair_contract", &amount);
//         let seed = to_binary(&"SEED".to_string())?;
//         let entropy = to_binary(&"ENTROPY".to_string())?;
//         let ref mut deps = mock_dependencies(30, &[]);
//         let mut env = mkenv("test");
//         env.block.height = 200_000;
//         env.contract.address = HumanAddr("ContractAddress".to_string());
//         let token_pair = mk_token_pair();
//         let msg = InitMsg {
//             pair: token_pair,
//             lp_token_contract: ContractInstantiationInfo{
//                   code_hash: "CODE_HASH".to_string(),
//                   id :0
//             },
//             factory_info: ContractLink {
//                 address: HumanAddr(String::from("FACTORYADDR")),
//                 code_hash: "FACTORYADDR_HASH".to_string()
//             },
//             prng_seed: seed.clone(),
//             entropy: entropy.clone(),
//             admin: Some(HumanAddr::from(env.message.sender.clone())),
//             callback: Some(Callback {
//                 contract: ContractLink {
//                     address: HumanAddr(String::from("CALLBACKADDR")),
//                     code_hash: "Test".to_string()
//                 },
//                 msg: to_binary(&String::from("Welcome bytes"))?
//             }),
//             staking_contract: None,
//             custom_fee: None
//         };     
//         assert!(init(deps, env.clone(), msg).is_ok());
      
 
//         let test_view_key = create_viewing_key(&env,seed.clone(),entropy.clone());
//         // load config
//         let config = load_config(deps).unwrap();
//         //assert_eq!("WETH".to_string(), config.symbol);
//         assert_eq!(test_view_key, config.viewing_key);
//         Ok(())
//     }

 
//     #[test]
//     fn assert_load_trade_history_first_time() -> StdResult<()>{
//         let deps = mkdeps();
//         let initial_value = load_trade_counter(&deps.storage)?;
//         assert_eq!(0, initial_value);
//         Ok(())
//     }

//     #[test]
//     fn assert_store_and_load_config_success() -> StdResult<()>{
//         let mut deps = mkdeps();
//         let token_pair = mk_token_pair();
//         let config = make_init_config(&mut deps,token_pair)?;   
//         store_config(&mut deps, config.clone())?;
//         let stored_config = load_config(&mut deps)?;
//         assert_eq!(config.pair.0, stored_config.pair.0);
//         Ok(())
//     }


//     #[test]
//     fn assert_store_trade_history_increase_counter_and_store_success()-> StdResult<()>{
//         let mut deps = mkdeps();
//         let env = mkenv("sender");       
//         let trade_history = TradeHistory {
//             price: "50".to_string(),
//             amount_in: Uint128::from(50u128),
//             amount_out: Uint128::from(50u128),
//             timestamp: 6000,
//             direction: "Sell".to_string(),
//             total_fee_amount: Uint128::from(50u128),
//             lp_fee_amount: Uint128::from(50u128),
//             shade_dao_fee_amount: Uint128::from(50u128),
//             height: 1045667,
//             trader: "address_hash".to_string()
//         };
//         store_trade_history(&mut deps, &trade_history)?;
//         let current_index = load_trade_counter(&deps.storage)?;
//         assert_eq!(1, current_index);

//         // load trade history
//         let stored_trade_history = load_trade_history(&deps, current_index)?;
//         assert_eq!(trade_history.price, stored_trade_history.price);
//         Ok(())
//     }

   
//     #[test]
//     fn assert_add_address_to_whitelist_success()-> StdResult<()>{
//         let mut deps = mkdeps();
//         let env = mkenv("sender");       
//         let addressA = HumanAddr::from("TESTA".to_string());
//         let addressB = HumanAddr::from("TESTB".to_string());
//         let addressC = HumanAddr::from("TESTC").to_string();
//         add_whitelist_address(&mut deps.storage, addressA.clone())?;
//         let current_index = load_whitelist_address(&deps.storage)?;
//         assert_eq!(1, current_index.len());        
//         add_whitelist_address(&mut deps.storage, addressB.clone())?;
//         let current_index = load_whitelist_address(&deps.storage)?;
//         assert_eq!(2, current_index.len());
//         Ok(())
//     }

//     //#[test]
//     fn assert_remove_address_from_whitelist_success()-> StdResult<()>{
//         let mut deps = mkdeps();
//         let env = mkenv("sender");       
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let address_b = HumanAddr::from("TESTB".to_string());
//         let address_c = HumanAddr::from("TESTC".to_string());
//         add_whitelist_address(&mut deps.storage, address_a.clone())?;
//         let current_index = load_whitelist_address(&deps.storage)?;
//         assert_eq!(1, current_index.len());        
//         add_whitelist_address(&mut deps.storage, address_b.clone())?;
//         let current_index = load_whitelist_address(&deps.storage)?;
//         assert_eq!(2, current_index.len());   
//         let mut list_addresses_remove  = Vec::new();
//         list_addresses_remove.push(address_b.clone());
//         remove_whitelist_address(&mut deps.storage, list_addresses_remove)?;
//         add_whitelist_address(&mut deps.storage, address_c.clone())?;
//         let current_index = load_whitelist_address(&deps.storage)?;
//         assert_eq!(2, current_index.len());        
//         Ok(())
//     }

    
//     #[test]
//     fn assert_load_address_from_whitelist_success()-> StdResult<()>{
//         let mut deps = mkdeps();
//         let env = mkenv("sender");       
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let address_b = HumanAddr::from("TESTB".to_string());
//         let address_c = HumanAddr::from("TESTC".to_string());
//         add_whitelist_address(&mut deps.storage, address_a.clone())?;
//         add_whitelist_address(&mut deps.storage, address_b.clone())?;
//         add_whitelist_address(&mut deps.storage, address_c.clone())?;
//         let stub_list = load_whitelist_address(&deps.storage)?;
//         assert_eq!(3, stub_list.len());
//         let is_addr = is_address_in_whitelist(&mut deps.storage, address_b.clone())?;  
//         assert_eq!(true, is_addr);      
//         let is_addr = is_address_in_whitelist(&mut deps.storage, HumanAddr("TESTD".to_string()).clone())?;  
//         assert_eq!(false, is_addr);   
//         Ok(())
//     }

//     #[test]
//     fn assert_swap_native_snip20()-> StdResult<()>{
//         let mut deps = mock_deps();
//         let env = mock_env(CONTRACT_ADDRESS, &[]);
//         let token_pair = mk_native_token_pair();
//         let config = make_init_config(&mut deps, token_pair.clone())?;       
//         let address_a = HumanAddr("TESTA".to_string());      
//         let msg = to_binary("Test")?;
//         assert_eq!(config.factory_info.address.as_str(), FACTORY_CONTRACT_ADDRESS.clone());
//         let native_swap = swap(&mut deps, env, config, address_a.clone(), 
//             None,  mk_custom_token_amount(Uint128::from(1000u128), token_pair.clone()),None, 
//             None, None)?;      
//         assert_eq!(native_swap.log[3].value, "997".to_string());
//         assert_eq!(native_swap.messages.len(), 1);
//         Ok(())
//     }

//     #[test]
//     fn assert_query_get_amm_pairs_success()-> StdResult<()>{
//         let mut deps = mkdeps();
//         let env = mkenv(MOCK_CONTRACT_ADDR);
//         let amm_settings = mk_amm_settings();
//         let token_pair = mk_token_pair();
//         let config = make_init_config(&mut deps, token_pair)?;         
//         let offer_amount: u128 = 34028236692093846346337460;
//         let expected_amount: u128 = 34028236692093846346337460;           
//         let address_a = HumanAddr::from("TESTA".to_string());
//         handle(
//             &mut deps,
//             env,
//             ExecuteMsg::AddWhiteListAddress {
//                 address: address_a.clone()
//             },
//         )
//         .unwrap();

//         let result = query(
//             &deps,
//             QueryMsg::GetWhiteListAddress {
//             },
//         )
//         .unwrap();

//         let response: QueryMsgResponse = from_binary(&result).unwrap();

//         match response {
//             QueryMsgResponse::GetWhiteListAddress { addresses: stored } => {
//                 assert_eq!(1, stored.len())
//             }
//             _ => panic!("QueryResponse::ListExchanges"),
//         }
//         Ok(())
//     }

    
// fn make_init_config<S: Storage, A: Api, Q: Querier>(
//     deps: DepsMut, 
//     token_pair: TokenPair<HumanAddr>) -> StdResult<Config<HumanAddr>> {    
//     let seed = to_binary(&"SEED".to_string())?;
//     let entropy = to_binary(&"ENTROPY".to_string())?;
//     let env = mkenv(MOCK_CONTRACT_ADDR);  
//     let msg = InitMsg {
//         pair: token_pair.clone(),
//         lp_token_contract: ContractInstantiationInfo{
//               code_hash: "CODE_HASH".to_string(),
//               id :0
//         },
//         factory_info: ContractLink {
//             address: HumanAddr(String::from(FACTORY_CONTRACT_ADDRESS)),
//             code_hash: "TEST".to_string()
//         },
//         prng_seed: seed.clone(),
//         entropy: entropy.clone(),
//         admin: Some(HumanAddr::from(env.message.sender.clone())),
//         callback: Some(Callback {
//             contract: ContractLink {
//                 address: HumanAddr(String::from("CALLBACKADDR")),
//                 code_hash: "Test".to_string()
//             },
//             msg: to_binary(&String::from("Welcome bytes"))?,
//         }),
//         staking_contract: None,
//         custom_fee: None
//     };         
//     assert!(init(deps, env.clone(), msg).is_ok());
//     let config = load_config(deps)?;
//     Ok(config)
// }

// fn mkdeps() -> Deps<impl Storage, impl Api, impl Querier> {
//     mock_dependencies(30, &[])
// }

// fn mk_token_pair() -> TokenPair<HumanAddr>{
//     let pair =  TokenPair(
//         TokenType::CustomToken {
//             contract_addr: HumanAddr(CUSTOM_TOKEN_1.to_string().clone()),
//             token_code_hash: CUSTOM_TOKEN_1.to_string()
//         },            
//         TokenType::CustomToken {
//             contract_addr: HumanAddr(CUSTOM_TOKEN_2.to_string().clone()),
//             token_code_hash: CUSTOM_TOKEN_2.to_string()
//         }
//     );
//     pair
// }

// fn mk_native_token_pair() -> TokenPair<HumanAddr>{
//     let pair =  TokenPair(
//         TokenType::CustomToken {
//             contract_addr: HumanAddr(CUSTOM_TOKEN_2.to_string()),
//             token_code_hash: CUSTOM_TOKEN_2.to_string()
//         },            
//         TokenType::NativeToken {
//             denom: "uscrt".into()
//         }
//     );
//     pair
// }


// fn mk_custom_token_amount(amount: Uint128, token_pair: TokenPair<HumanAddr>) -> TokenAmount<HumanAddr>{    
//     let token = TokenAmount{
//         token: token_pair.0.clone(),
//         amount: amount.clone(),
//     };
//     token
// }

// fn mk_custom_token(address: String) -> TokenType<HumanAddr>{
//     TokenType::CustomToken {
//         contract_addr: HumanAddr(address.clone()),
//         token_code_hash: "TOKEN0_HASH".to_string()
//     }
// }

// fn mk_native_token() -> TokenType<HumanAddr>{
//     TokenType::NativeToken{
//         denom: "uscrt".to_string()
//     }
// }

// fn mk_amm_settings() -> AMMSettings<HumanAddr>{
//     AMMSettings{
//         lp_fee: Fee{
//             nom: 2,
//             denom: 100
//         },
//         shade_dao_fee: Fee {
//             nom: 1,
//             denom: 100
//         },
//         shade_dao_address: ContractLink{
//             code_hash: "CODEHAS".to_string(),
//             address: HumanAddr("TEST".to_string())
//         }
//     }
// }

// fn mock_config(env: Env) -> StdResult<Config<HumanAddr>>
// {    
//     let seed = to_binary(&"SEED".to_string())?;
//     let entropy = to_binary(&"ENTROPY".to_string())?;

//     Ok(Config {       
//         factory_info: mock_contract_link(FACTORY_CONTRACT_ADDRESS.to_string()),
//         lp_token_info: mock_contract_link("LPTOKEN".to_string()),
//         pair:      mk_token_pair(),
//         contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
//         viewing_key:  create_viewing_key(&env, seed.clone(), entropy.clone()),
//         custom_fee: None
//     })
// }

// pub fn mock_contract_link(address: String)-> ContractLink{
//     ContractLink{
//         address: HumanAddr::from(address.clone()),
//         code_hash: "CODEHASH".to_string()
//     }
// }

// fn mock_contract_info(address: &str) -> ContractInfo{
//     ContractInfo{
//         address :HumanAddr::from(address.clone())
//     }
// }


// fn mock_deps() -> Deps<MockStorage, MockApi, MockQuerier> {
//     Extern {
//         storage: MockStorage::default(),
//         api: MockApi::new(123),
//         querier: MockQuerier { portion: 2500 },
//     }
// }

// struct MockQuerier{
//     portion: u128,
// }

// impl Querier for MockQuerier {
//     fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
//         let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
//         match &request {
//             QueryRequest::Wasm(msg) => {
//                 match msg {
//                     WasmQuery::Smart { contract_addr, .. } => {
//                         println!("Factory Address :: {}", contract_addr);
//                         match contract_addr.as_str() {
//                             FACTORY_CONTRACT_ADDRESS => {
//                                 let amm_settings = shadeswap_shared::amm_pair::AMMSettings {
//                                     lp_fee: Fee::new(28, 10000),
//                                     shade_dao_fee: Fee::new(2, 10000),
//                                     shade_dao_address: ContractLink {
//                                         address: HumanAddr(String::from("DAO")),
//                                         code_hash: "".to_string(),
//                                     }
//                                 };
//                                 let response = FactoryQueryResponse::GetAMMSettings {
//                                     settings: amm_settings
//                                 };
//                                 QuerierResult::Ok(to_binary(&response))
//                             },
//                             CUSTOM_TOKEN_2 => {                                
//                                 QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                     balance: Balance {
//                                         amount: Uint128(1000000),
//                                     },
//                                 }))
//                             },
//                             CUSTOM_TOKEN_1 =>{
//                                 QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                     balance: Balance{
//                                         amount: Uint128(1000000),
//                                     },
//                                 }))
//                             },                        
//                             CONTRACT_ADDRESS => {
//                                 QuerierResult::Ok(to_binary(&BalanceResponse{
//                                     amount: Coin{
//                                         denom: "uscrt".into(),
//                                         amount: Uint128(1000000u128),
//                                     }
//                                 }))
//                             }
//                             _ => unimplemented!()
//                         }
//                     },                  
//                     _ => unimplemented!(),
//                 }
//             },      
//             QueryRequest::Bank(msg) => {
//                 match msg {
//                     BankQuery::Balance {address, .. } => {
//                         println!("Factory Address :: {}", address);
//                         match address.as_str() {
//                             CONTRACT_ADDRESS => {
//                                 QuerierResult::Ok(to_binary(&BalanceResponse{
//                                     amount: Coin{
//                                         denom: "uscrt".into(),
//                                         amount: Uint128(1000000u128),
//                                     }
//                                 }))
//                             }, 
//                             "cosmos2contract" => {
//                                 QuerierResult::Ok(to_binary(&BalanceResponse{
//                                     amount: Coin{
//                                         denom: "uscrt".into(),
//                                         amount: Uint128(1000000u128),
//                                     }
//                                 }))
//                             },                          
//                             _ => {                            
//                                 unimplemented!()
//                             } 
//                         }
//                     },
//                     _ => unimplemented!(),
//                 }
//             },  
//             _ => unimplemented!(),
//         }
//     }

//     fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
//         self.custom_query(request)
//     }

//     fn custom_query<T: serde::Serialize, U: DeserializeOwned>(
//         &self,
//         request: &QueryRequest<T>,
//     ) -> StdResult<U> {
//         let raw = match to_vec(request) {
//             Ok(raw) => raw,
//             Err(e) => {
//                 return Err(StdError::generic_err(format!(
//                     "Serializing QueryRequest: {}",
//                     e
//                 )))
//             }
//         };
//         match self.raw_query(&raw) {
//             Err(sys) => Err(StdError::generic_err(format!(
//                 "Querier system error: {}",
//                 sys
//             ))),
//             Ok(Err(err)) => Err(err),
//             // in theory we would process the response, but here it is the same type, so just pass through
//             Ok(Ok(res)) => from_binary(&res),
//         }
//     }

//     fn query_balance<U: Into<HumanAddr>>(&self, address: U, denom: &str) -> StdResult<Coin> {
//         let request = BankQuery::Balance {
//             address: address.into(),
//             denom: denom.to_string(),
//         }
//         .into();
//         let res: BalanceResponse = self.query(&request)?;
//         Ok(res.amount)
//     }

//     fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
//         let request = BankQuery::AllBalances {
//             address: address.into(),
//         }
//         .into();
//         let res: AllBalanceResponse = self.query(&request)?;
//         Ok(res.amount)
//     }
// }

// #[derive(Serialize, Deserialize)]
// struct IntBalanceResponse {
//     pub balance: Balance,
// }

// }


// #[cfg(test)]
// pub mod tests_calculation_price_and_fee{
//     use super::*;
//     use cosmwasm_std::Decimal;
//     use serde::de::DeserializeOwned;
//     use shadeswap_shared::custom_fee::{CustomFee, Fee};
//     use crate::contract::set_staking_contract;
//     use crate::state::amm_pair_storage::{{store_trade_history, load_trade_history, load_trade_counter}};
//     use crate::state::tradehistory::{{ DirectionType}};  
//     use serde::Deserialize;
//     use serde::Serialize;
//     use crate::test::help_test_lib::{{mock_deps_with_expected_return_value, mk_amm_settings_a, mkenv, 
//         mk_custom_token_amount_test_calculation_price_fee,
//         mk_token_pair_test_calculation_price_fee, make_init_config_test_calculate_price_fee,
//         mk_native_token_pair_test_calculation_price_fee,  mock_deps_for_slippage_test}};
    
//     #[test]
//     fn assert_calculate_and_print_price() -> StdResult<()>{
//         let result_a = calculate_and_print_price(Uint128(99), Uint128(100),0)?;
//         let result_b = calculate_and_print_price(Uint128(58), Uint128(124),1)?;
//         let result_c = calculate_and_print_price(Uint128(158), Uint128(124),0)?;
//         assert_eq!(result_a, "0.99".to_string());
//         assert_eq!(result_b, "0.467741935".to_string());
//         assert_eq!(result_c, "1.274193548387096774".to_string());
//         Ok(())
//     }

//     #[test]
//     fn assert_calculate_price() -> StdResult<()>{     
//         let price = calculate_price(Uint128(2000), Uint128(10000), Uint128(100000));
//         assert_eq!(Uint128(16666), price?);
//         Ok(())
//     }

//     #[test]
//     fn assert_calculate_price_sell() -> StdResult<()>{     
//         let price = calculate_price(Uint128(2000), Uint128(100000), Uint128(10000));
//         assert_eq!(Uint128(196), price?);
//         Ok(())
//     }

    
//     #[test]
//     fn assert_initial_swap_with_token_success_without_fee() -> StdResult<()>
//     {     
//         let custom_fee: Option<CustomFee> = None;
//         let mut deps = mock_deps_with_expected_return_value();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, custom_fee)?;           
//         let offer_amount: u128 = 2000;
//         let expected_amount: u128 = 16666;
//         let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, 
//             &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
//             &mut deps.storage, HumanAddr("Test".to_string().clone()), Some(true));
//         assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
//         Ok(())
//     }

//     #[test]
//     fn assert_initial_swap_with_token_success_with_fee() -> StdResult<()>
//     {     
//         let custom_fee: Option<CustomFee> = None;
//         let mut deps = mock_deps_with_expected_return_value();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, custom_fee)?;           
//         let offer_amount: u128 = 2000;
//         let expected_amount: u128 = 16247;
//         let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, 
//             &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
//             &mut deps.storage, HumanAddr("Test".to_string().clone()), None);
//         assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
//         Ok(())
//     }

    
//     #[test]
//     fn assert_swap_with_custom_fee_success() -> StdResult<()>{
//         let custom_fee = Some( CustomFee{
//             shade_dao_fee: Fee { nom: 8, denom: 100 },
//             lp_fee: Fee { nom: 1, denom: 100},
//         });
//         let mut deps = mock_deps_with_expected_return_value();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, custom_fee)?;           
//         let offer_amount: u128 = 2000;
//         let expected_amount: u128 = 15397;
//         let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, 
//             &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
//             &mut deps.storage, HumanAddr("Test".to_string().clone()), None);
//         assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
//         Ok(())
//     }

    
//     #[test]
//     fn assert_calculate_swap_result_without_custom_fee() -> StdResult<()>{
//         let custom_fee: Option<CustomFee> = None;
//         let mut deps = mock_deps_with_expected_return_value();
//         let token_pair = mk_native_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair.clone(), None)?;       
//         let address_a = HumanAddr("TESTA".to_string());
//         let token_amount = mk_custom_token_amount_test_calculation_price_fee(Uint128(2000), config.pair.clone());   
//         let amm_settings = shadeswap_shared::amm_pair::AMMSettings {
//             lp_fee: Fee::new(2, 100),
//             shade_dao_fee: Fee::new(3, 100),
//             shade_dao_address: ContractLink {
//                 address: HumanAddr(String::from("DAO")),
//                 code_hash: "".to_string(),
//             }
//         };
//         assert_eq!(config.factory_info.address.as_str(), FACTORY_CONTRACT_ADDRESS.clone());
//         let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, &token_amount,
//             &mut deps.storage, address_a, None)?;
//         assert_eq!(swap_result.result.return_amount, Uint128(186u128));
//         assert_eq!(swap_result.lp_fee_amount, Uint128(40u128));
//         assert_eq!(swap_result.shade_dao_fee_amount, Uint128(60u128));
//         assert_eq!(swap_result.price, "0.093".to_string());
//         Ok(())
//     }

//     #[test]
//     fn assert_initial_swap_with_zero_fee_for_whitelist_address()-> StdResult<()>{
//         let mut deps = mock_deps_with_expected_return_value();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, None)?;         
//         let offer_amount: u128 = 2000;
//         let expected_amount: u128 = 16666;           
//         let address_a = HumanAddr::from("TESTA".to_string());
//         add_whitelist_address(&mut deps.storage, address_a.clone())?;    
//         let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, 
//             &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
//             &mut deps.storage, address_a.clone(), None)?;
//         assert_eq!(Uint128::from(expected_amount), swap_result.result.return_amount);
//         assert_eq!(Uint128::zero(), swap_result.lp_fee_amount);
//         Ok(())
//     }


//     #[test]
//     fn assert_slippage_swap_result_with_less_return_amount_throw_exception() -> StdResult<()>{
//         let mut deps = mock_deps_for_slippage_test();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, None)?;         
//         let offer_amount: u128 = 2000;
//         let expected_amount: u128 = 16666;           
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let token = config.pair.clone();        
//         let swap_and_test_slippage = swap(
//             & mut deps,
//             mkenv(address_a.clone()),
//             config,
//             address_a.clone(),
//             Some(address_a.clone()),          
//             mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), token), 
//             Some(Uint128(40000)),
//             None, 
//             None
//         );

//         match swap_and_test_slippage.unwrap_err() {
//             e =>  assert_eq!(e, StdError::generic_err(
//                 "Operation fell short of expected_return",
//             )),
//         }       
//         Ok(())
//     }

//     #[test]
//     fn assert_slippage_swap_result_with_higher_return_amount_success() -> StdResult<()>{
//         let mut deps = mock_deps_for_slippage_test();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair, None)?;         
//         let offer_amount: u128 = 2000;          
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let token = config.pair.clone();   
//         let swap_and_test_slippage = swap(
//             & mut deps,
//             mkenv(address_a.clone()),
//             config,
//             address_a.clone(),
//             Some(address_a.clone()),          
//             mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), token), 
//             Some(Uint128(400)),
//             None, 
//             None
//         );

//         assert_eq!(
//             swap_and_test_slippage.unwrap().log[3], 
//             log("return_amount", Uint128(15397)));
//         Ok(())
//     }

//     #[test]
//     fn assert_slippage_add_liqudity_with_wrong_ration_throw_error() -> StdResult<()>{
//         let mut deps = mock_deps_for_slippage_test();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair.clone(), None)?;         
//         let offer_amount: u128 = 2000;          
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let token = config.pair.clone();  
//         let add_liquidity_with_err = add_liquidity(
//             &mut deps,
//             mkenv(address_a.clone()),
//             TokenPairAmount{
//                 pair: token_pair.clone(),
//                 amount_0: Uint128(1000000),
//                 amount_1: Uint128(10000)
//             },
//             Some(Decimal::percent(20)),
//             None
//         );       

//         match add_liquidity_with_err.unwrap_err() {
//             e =>  assert_eq!(e, StdError::generic_err(
//                 "Operation exceeds max slippage acceptance",
//             )),
//         }       
//         Ok(())
//     }

//     #[test]
//     fn assert_slippage_add_liqudity_with_right_ration_success() -> StdResult<()>{
//         let mut deps = mock_deps_for_slippage_test();
//         let amm_settings = mk_amm_settings_a();
//         let token_pair = mk_token_pair_test_calculation_price_fee();
//         let env = mkenv("address_a".to_string());
//         let config = make_init_config_test_calculate_price_fee(&mut deps, token_pair.clone(), None)?;         
//         // set lp token contract for liqudity
//         let mut config = load_config(&deps)?;
//         config.lp_token_info = ContractLink{ 
//             address: HumanAddr::from(LP_TOKEN_ADDRESS),
//             code_hash: "CODE_HASH".to_string()
//         };
//         store_config(&mut deps, config.clone())?;
//         let offer_amount: u128 = 2000;          
//         let address_a = HumanAddr::from("TESTA".to_string());
//         let token = config.pair.clone();  
//         let add_liquidity_with_success = add_liquidity(
//             &mut deps,
//             env.clone(),
//             TokenPairAmount{
//                 pair: token_pair.clone(),
//                 amount_0: Uint128(10000),
//                 amount_1: Uint128(100000)
//             },
//             Some(Decimal::percent(20)),
//             None
//         );       

//         assert_eq!(
//             add_liquidity_with_success.unwrap().log[3], 
//             log("share_pool", Uint128(10000)));    
//         Ok(())
//     }
// }


// pub mod help_test_lib {
//     use super::*;       
//     use shadeswap_shared::{token_amount::{{TokenAmount}}, core::{ContractInstantiationInfo, Callback}};
//     use shadeswap_shared::token_pair::{{TokenPair}};
//     use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
//     use shadeswap_shared::token_type::{{TokenType}};
//     use shadeswap_shared::amm_pair::{{AMMPair, AMMSettings}};
//     use crate::{state::{Config}, contract::set_staking_contract};
//     use crate::state::amm_pair_storage::{{ store_config, load_config,
//         remove_whitelist_address,is_address_in_whitelist, add_whitelist_address,load_whitelist_address, }};
//     use crate::contract::init;
//     use crate::contract::{{create_viewing_key, calculate_price, calculate_swap_result,swap, query, handle}};
//     use std::hash::Hash;
//     use serde::de::DeserializeOwned;
//     use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
//      use crate::state::amm_pair_storage::{{store_trade_history, load_trade_history, load_trade_counter}};
//     use crate::state::tradehistory::{{ DirectionType}};  
//     use serde::Deserialize;
//     use serde::Serialize;
//     use crate::help_math::calculate_and_print_price;
//     use shadeswap_shared::custom_fee::{Fee, CustomFee};

 
//     pub fn mock_deps_with_expected_return_value() -> Deps<MockStorage, MockApi, MockQuerierExpectedValue> {
//         Extern {
//             storage: MockStorage::default(),
//             api: MockApi::new(123),
//             querier: MockQuerierExpectedValue { portion: 2500 },
//         }
//     }
    
//     pub struct MockQuerierExpectedValue{
//         portion: u128,
//     }
    
//     impl Querier for MockQuerierExpectedValue{
//         fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
//             let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
//             match &request {
//                 QueryRequest::Wasm(msg) => {
//                     match msg {
//                         WasmQuery::Smart { contract_addr, .. } => {
//                             println!("Factory Address :: {}", contract_addr);
//                             match contract_addr.as_str() {                          
//                                 CUSTOM_TOKEN_2 => {                                
//                                     QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                         balance: Balance {
//                                             amount: Uint128(100000),
//                                         },
//                                     }))
//                                 },
//                                 CUSTOM_TOKEN_1 =>{
//                                     QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                         balance: Balance{
//                                             amount: Uint128(10000),
//                                         },
//                                     }))
//                                 },    
//                                 _ => unimplemented!()
//                             }
//                         },                  
//                         _ => unimplemented!(),
//                     }
//                 },      
//                 QueryRequest::Bank(msg) => {
//                     match msg {
//                         BankQuery::Balance {address, .. } => {
//                             println!("Factory Address :: {}", address);
//                             match address.as_str() {
//                                 CONTRACT_ADDRESS => {
//                                     QuerierResult::Ok(to_binary(&BalanceResponse{
//                                         amount: Coin{
//                                             denom: "uscrt".into(),
//                                             amount: Uint128(10000u128),
//                                         }
//                                     }))
//                                 }, 
//                                 "cosmos2contract" => {
//                                     QuerierResult::Ok(to_binary(&BalanceResponse{
//                                         amount: Coin{
//                                             denom: "uscrt".into(),
//                                             amount: Uint128(10000u128),
//                                         }
//                                     }))
//                                 },                          
//                                 _ => {                            
//                                     unimplemented!()
//                                 } 
//                             }
//                         },
//                         _ => unimplemented!(),
//                     }
//                 },  
//                 _ => unimplemented!(),
//             }
//         }
    
//         fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
//             self.custom_query(request)
//         }
    
//         fn custom_query<T: serde::Serialize, U: DeserializeOwned>(
//             &self,
//             request: &QueryRequest<T>,
//         ) -> StdResult<U> {
//             let raw = match to_vec(request) {
//                 Ok(raw) => raw,
//                 Err(e) => {
//                     return Err(StdError::generic_err(format!(
//                         "Serializing QueryRequest: {}",
//                         e
//                     )))
//                 }
//             };
//             match self.raw_query(&raw) {
//                 Err(sys) => Err(StdError::generic_err(format!(
//                     "Querier system error: {}",
//                     sys
//                 ))),
//                 Ok(Err(err)) => Err(err),
//                 // in theory we would process the response, but here it is the same type, so just pass through
//                 Ok(Ok(res)) => from_binary(&res),
//             }
//         }
    
//     }
    
    
//     pub fn mock_deps_for_slippage_test() -> Deps<MockStorage, MockApi, MockQuerierSlippageValue> {
//         Extern {
//             storage: MockStorage::default(),
//             api: MockApi::new(123),
//             querier: MockQuerierSlippageValue { portion: 2500 },
//         }
//     }
    
    
//     pub fn mk_amm_settings_a() -> AMMSettings<HumanAddr>{
//         AMMSettings{
//             lp_fee: Fee{
//                 nom: 2,
//                 denom: 100
//             },
//             shade_dao_fee: Fee {
//                 nom: 1,
//                 denom: 100
//             },
//             shade_dao_address: ContractLink{
//                 code_hash: "CODEHAS".to_string(),
//                 address: HumanAddr("TEST".to_string())
//             }
//         }
//     }
    
//     pub struct MockQuerierSlippageValue{
//         portion: u128,
//     }
    
//     #[derive(Serialize, Deserialize,  PartialEq, Debug)]
//     pub struct TokenInfoResponseMock {
//         pub token_info: TokenInfo,
//     }
    
//     impl Querier for MockQuerierSlippageValue{
//         fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
//             let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
//             match &request {
//                 QueryRequest::Wasm(msg) => {
//                     match msg {
//                         WasmQuery::Smart { contract_addr, .. } => {
//                             println!("Factory Address :: {}", contract_addr);
//                             match contract_addr.as_str() {                          
//                                 CUSTOM_TOKEN_2 => {                                
//                                     QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                         balance: Balance {
//                                             amount: Uint128(100000),
//                                         },
//                                     }))
//                                 },
//                                 CUSTOM_TOKEN_1 =>{
//                                     QuerierResult::Ok(to_binary(&IntBalanceResponse {
//                                         balance: Balance{
//                                             amount: Uint128(10000),
//                                         },
//                                     }))
//                                 },    
//                                 FACTORY_CONTRACT_ADDRESS => {
//                                     QuerierResult::Ok(to_binary(&FactoryQueryResponse::GetAMMSettings{
//                                         settings: AMMSettings { 
//                                             lp_fee: Fee { nom: 1, denom: 100}, 
//                                             shade_dao_fee: Fee {nom: 8, denom: 100}, 
//                                             shade_dao_address:  ContractLink { 
//                                                 address: HumanAddr::from("shade_dao_address".to_string()),
//                                                 code_hash: "CODE_HASH".to_string(),
//                                             }
//                                         }
//                                     }))
//                                 },
//                                 P_TOKEN_ADDRESS => {
//                                     QuerierResult::Ok(to_binary(&TokenInfoResponseMock{
//                                             token_info: TokenInfo {
//                                             name: "LPTOKEN".to_string(),
//                                             decimals: 18,
//                                             symbol: "LP".to_string(),
//                                             total_supply: Some(Uint128(10000))
//                                         }
//                                     }))
//                                 },
//                                 _ => {
//                                     println!("{}", contract_addr.as_str()); 
//                                     return unimplemented!() 
//                                 }
//                             }
//                         },                  
//                         _ => unimplemented!(),
//                     }
//                 },      
//                 QueryRequest::Bank(msg) => {
//                     match msg {
//                         BankQuery::Balance {address, .. } => {
//                             println!("Factory Address :: {}", address);
//                             match address.as_str() {
//                                 CONTRACT_ADDRESS => {
//                                     QuerierResult::Ok(to_binary(&BalanceResponse{
//                                         amount: Coin{
//                                             denom: "uscrt".into(),
//                                             amount: Uint128(10000u128),
//                                         }
//                                     }))
//                                 }, 
//                                 "cosmos2contract" => {
//                                     QuerierResult::Ok(to_binary(&BalanceResponse{
//                                         amount: Coin{
//                                             denom: "uscrt".into(),
//                                             amount: Uint128(10000u128),
//                                         }
//                                     }))
//                                 },                          
//                                 _ => {                            
//                                     unimplemented!()
//                                 } 
//                             }
//                         },
//                         _ => unimplemented!(),
//                     }
//                 },  
//                 _ => unimplemented!(),
//             }
//         }
    
//         fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
//             self.custom_query(request)
//         }
    
//         fn custom_query<T: serde::Serialize, U: DeserializeOwned>(
//             &self,
//             request: &QueryRequest<T>,
//         ) -> StdResult<U> {
//             let raw = match to_vec(request) {
//                 Ok(raw) => raw,
//                 Err(e) => {
//                     return Err(StdError::generic_err(format!(
//                         "Serializing QueryRequest: {}",
//                         e
//                     )))
//                 }
//             };
//             match self.raw_query(&raw) {
//                 Err(sys) => Err(StdError::generic_err(format!(
//                     "Querier system error: {}",
//                     sys
//                 ))),
//                 Ok(Err(err)) => Err(err),
//                 // in theory we would process the response, but here it is the same type, so just pass through
//                 Ok(Ok(res)) => from_binary(&res),
//             }
//         }   
//     }
    
//     #[derive(Serialize, Deserialize)]
//     pub struct IntBalanceResponse {
//         pub balance: Balance,
//     }
    
//     pub fn mk_custom_token_amount_test_calculation_price_fee(amount: Uint128, token_pair: TokenPair<HumanAddr>) -> TokenAmount<HumanAddr>{    
//         let token = TokenAmount{
//             token: token_pair.0.clone(),
//             amount: amount.clone(),
//         };
//         token
//     }
    
//     pub fn mk_native_token_pair_test_calculation_price_fee() -> TokenPair<HumanAddr>{
//         let pair =  TokenPair(
//             TokenType::CustomToken {
//                 contract_addr: HumanAddr(CUSTOM_TOKEN_2.to_string()),
//                 token_code_hash: CUSTOM_TOKEN_2.to_string()
//             },            
//             TokenType::NativeToken {
//                 denom: "uscrt".into()
//             }
//         );
//         pair
//     }
    
//     // pub fn mock_config_test_calculation_price_fee(env: Env) -> StdResult<Config<HumanAddr>> {
//     // {    
//     //     let seed = to_binary(&"SEED".to_string())?;
//     //     let entropy = to_binary(&"ENTROPY".to_string())?;
    
//     //     let config = Config {       
//     //         factory_info: mock_contract_link(FACTORY_CONTRACT_ADDRESS.to_string()),
//     //         lp_token_info: mock_contract_link("LPTOKEN".to_string()),
//     //         pair:      mk_token_pair_test_calculation_price_fee(),
//     //         contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
//     //         viewing_key:  create_viewing_key(&env, seed.clone(), entropy.clone()),
//     //         custom_fee: None
//     //     };
//     //     Ok(config)
//     // }
    
//     pub fn make_init_config_test_calculate_price_fee<S: Storage, A: Api, Q: Querier>(
//         deps: DepsMut, 
//         token_pair: TokenPair<HumanAddr>,
//         custom_fee: Option<CustomFee>,
//     ) 
//     -> StdResult<Config<HumanAddr>> {    
//         let seed = to_binary(&"SEED".to_string())?;
//         let entropy = to_binary(&"ENTROPY".to_string())?;
//         let env = mkenv(CONTRACT_ADDRESS);  
//         let msg = InitMsg {
//             pair: token_pair.clone(),
//             lp_token_contract: ContractInstantiationInfo{
//                   code_hash: "CODE_HASH".to_string(),
//                   id :0
//             },
//             factory_info: ContractLink {
//                 address: HumanAddr(String::from(FACTORY_CONTRACT_ADDRESS)),
//                 code_hash: "TEST".to_string()
//             },
//             prng_seed: seed.clone(),
//             entropy: entropy.clone(),
//             admin: Some(HumanAddr::from(env.message.sender.clone())),
//             callback: Some(Callback {
//                 contract: ContractLink {
//                     address: HumanAddr(String::from("CALLBACKADDR")),
//                     code_hash: "Test".to_string()
//                 },
//                 msg: to_binary(&String::from("Welcome bytes"))?,
//             }),
//             staking_contract: None,
//             custom_fee: custom_fee,
//         };         
//         assert!(init(deps, env.clone(), msg).is_ok());
//         let config = load_config(deps)?;    // set staking contract
        
//         Ok(config)
//     }
    
//     pub fn mkenv(sender: impl Into<HumanAddr>) -> Env {
//         mock_env(sender, &[])
//     }
    
//     pub fn mk_token_pair_test_calculation_price_fee() -> TokenPair<HumanAddr>{
//         let pair =  TokenPair(
//             TokenType::CustomToken {
//                 contract_addr: HumanAddr(CUSTOM_TOKEN_1.to_string().clone()),
//                 token_code_hash: CUSTOM_TOKEN_1.to_string()
//             },            
//             TokenType::CustomToken {
//                 contract_addr: HumanAddr(CUSTOM_TOKEN_2.to_string().clone()),
//                 token_code_hash: CUSTOM_TOKEN_2.to_string()
//             }
//         );
//         pair
//     }
    
//     pub fn mock_contract_link(address: String)-> ContractLink{
//         ContractLink{
//             address: HumanAddr::from(address.clone()),
//             code_hash: "CODEHASH".to_string()
//         }
//     }
    
//     fn mock_contract_info(address: &str) -> ContractInfo{
//         ContractInfo{
//             address :HumanAddr::from(address.clone())
//         }
//     }
    
// }
