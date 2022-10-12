// use amm_pair::contract::{execute, instantiate, query, reply};
// use snip20_reference_impl::contract::{execute as snip20_execute, instantiate as snip20_instantiate, query as  snip20_query};
// use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};
// use lp_token::contract::{execute as lp_execute, instantiate as lp_instantiate, query as lp_query};
// use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
// use shadeswap_shared::{
//     msg::amm_pair::{{QueryMsg, QueryMsgResponse}},
//     core::{ContractInstantiationInfo, ContractLink},
//     c_std::{QueryRequest, WasmQuery},
//     factory::{InitMsg as FactoryInitMsg, QueryResponse as FactoryQueryResponse, QueryMsg as FactoryQueryMsg}, 
//     utils::testing::TestingExt
// };
// use shadeswap_shared::msg::amm_pair::{{InitMsg}};
// use crate::{integration_help_lib::{mk_contract_link, mk_address}};
// use cosmwasm_std::{
//     testing::{mock_env, MockApi},
//     to_binary, Addr, Empty, Binary, ContractInfo,
// };


// pub fn amm_pair_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
//     Box::new(contract)
// } 

// pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query).with_reply(reply);
//     Box::new(contract)
// } 

// pub fn snip20_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new_with_empty(snip20_execute, snip20_instantiate, snip20_query).with_reply(reply);
//     Box::new(contract)
// } 

// pub fn factory_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(snip20_execute, snip20_instantiate, snip20_query).with_reply(reply);
//     Box::new(contract)
// } 

// pub fn lp_token_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(lp_execute, lp_instantiate, lp_query); //.with_reply(reply);
//     Box::new(contract)
// } 

// pub const CONTRACT_ADDRESS: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy6";
// pub const TOKEN_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
// pub const TOKEN_B: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy4";
// pub const FACTORY: &str = "secret13q9rgw3ez5mf808vm6k0naye090hh0m5fe2436";
// pub const OWNER: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";

// #[cfg(not(target_arch = "wasm32"))]
// #[test]
// pub fn amm_pair_integration_tests() {
//     use cosmwasm_std::Uint128;
//     use shadeswap_shared::{core::{TokenType, TokenPair}, snip20::{InstantiateMsg, InitConfig}, stake_contract::StakingContractInit};

//     use crate::integration_help_lib::generate_snip20_contract;   
       
//     let mut router = App::default();   

//     let factory_contract_link = ContractLink{
//         address: mk_address(FACTORY),
//         code_hash: "".to_string(),
//     };    
  
//     let amm_pair_contract_code_id = router.store_code(amm_pair_contract_store());     
//     let eth_snip20_contract = generate_snip20_contract(&mut router,  "ETH".to_string(),"ETH".to_string(),18);
//     let btc_snip20_contract = generate_snip20_contract(&mut router, "BTC".to_string(),"BTC".to_string(),18);
//     let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18);
//     let snip20_contract_code_id = router.store_code(snip20_contract_store());
//     let staking_contract = router.store_code(staking_contract_store());
//     let lptoken_contract_code_id = router.store_code(lp_token_contract_store());

//     let token_pair = TokenPair(
//         TokenType::CustomToken { contract_addr: eth_snip20_contract.address.to_owned(), token_code_hash: eth_snip20_contract.code_hash.to_owned() },
//         TokenType::CustomToken { contract_addr: btc_snip20_contract.address.to_owned(), token_code_hash: btc_snip20_contract.code_hash.to_owned() }
//     );

//     let init_msg = InitMsg {
//         pair: token_pair,
//         lp_token_contract: ContractInstantiationInfo { code_hash: lptoken_contract_code_id.code_hash.to_owned(), id: lptoken_contract_code_id.code_id },
//         factory_info: factory_contract_link.to_owned(),
//         prng_seed: to_binary(&"password").unwrap(),
//         entropy: to_binary(&"password").unwrap(),
//         admin: Some(mk_address(&OWNER)),
//         staking_contract: Some(StakingContractInit{ 
//             contract_info:  ContractInstantiationInfo { code_hash: staking_contract.code_hash.to_owned(), id: staking_contract.code_id},
//             amount: Uint128::new(10000), 
//             reward_token: TokenType::CustomToken { contract_addr: reward_contract.address.to_owned(), token_code_hash: reward_contract.code_hash }
//          }),
//         // staking_contract: None,
//         custom_fee: None,
//         callback: None,
//     };

//     let mocked_contract_addr = router
//         .instantiate_contract(
//             amm_pair_contract_code_id,
//             mk_address(&OWNER).to_owned(),
//             &init_msg,
//             &[],
//             "amm_pair",
//             None,
//         )
//         .unwrap();

//     println!("{}", mocked_contract_addr.address.to_string());
//     let query: QueryMsgResponse = router.query_test(mocked_contract_addr,to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
//     match query {
//         QueryMsgResponse::GetConfig { factory_contract, lp_token, staking_contract, pair, custom_fee } => {
//             assert_eq!(staking_contract, None);
//             assert_eq!(factory_contract, mk_contract_link(""));
//         },
//         _ => panic!("Query Responsedoes not match")
//     }
// }

// pub mod integration_help_lib{   
//     use cosmwasm_std::{Addr, ContractInfo};
//     use secret_multi_test::{App, Executor};
//     use shadeswap_shared::{msg::amm_pair::InitMsg, core::TokenPair, core::{TokenType, ContractLink}, snip20::{InitConfig, InstantiateMsg}};
//     use crate::{{TOKEN_A, TOKEN_B}, OWNER, snip20_contract_store};      
//     use cosmwasm_std::to_binary;

//     pub fn mk_token_pair() -> TokenPair{
//         return TokenPair(
//             TokenType::CustomToken { contract_addr: mk_address(TOKEN_A), token_code_hash: "".to_string() },
//             TokenType::CustomToken { contract_addr: mk_address(TOKEN_B), token_code_hash: "".to_string() }
//         );
//     }

//     pub fn mk_address(address: &str) -> Addr{
//         return Addr::unchecked(address.to_string())
//     }

//     pub fn mk_contract_link(address: &str) -> ContractLink{
//         return ContractLink{
//             address: mk_address(address),
//             code_hash: "".to_string(),
//         }       
//     }
    
//     pub fn generate_snip20_contract(
//         router: &mut App, 
//         name: String, 
//         symbol: String, 
//         decimal: u8) -> ContractInfo {

//         let snip20_contract_code_id = router.store_code(snip20_contract_store());        
//         let init_snip20_msg = InstantiateMsg {
//             name: name.to_owned(),
//             admin: Some(OWNER.to_string()),
//             symbol: symbol.to_owned(),
//             decimals: decimal,
//             initial_balances: None,
//             prng_seed: to_binary("password").unwrap(),
//             config: Some(InitConfig {
//                 public_total_supply: Some(true),
//                 enable_deposit: Some(false),
//                 enable_redeem: Some(false),
//                 enable_mint: Some(true),
//                 enable_burn: Some(true),
//                 enable_transfer: Some(true),
//             }),
//             query_auth: None,
//         };
//         let init_snip20_code_id = router
//             .instantiate_contract(
//                 snip20_contract_code_id,
//                 mk_address(&OWNER).to_owned(),
//                 &init_snip20_msg,
//                 &[],
//                 "token_a",
//                 None,
//             )
//         .unwrap();
//     init_snip20_code_id
// }
// }