use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, HandleMsg,SwapInfo, SwapResult,  InvokeMsg, QueryMsgResponse}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair::{{TokenPair}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use shadeswap_shared::amm_pair::{{AMMPair, AMMSettings}};
use crate::state::{Config};
use shadeswap_shared::msg::amm_pair::{{ TradeHistory}};
use crate::state::amm_pair_storage::{{ store_config, load_config,
    remove_whitelist_address,is_address_in_whitelist, add_whitelist_address,load_whitelist_address, }};
use crate::contract::init;
use shadeswap_shared::fadroma::secret_toolkit::snip20::Balance;
use crate::contract::{{create_viewing_key, calculate_price, calculate_swap_result,swap, query, handle}};
use std::hash::Hash;
use serde::de::DeserializeOwned;
use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
use shadeswap_shared::fadroma::Empty;
use shadeswap_shared::fadroma::from_slice;
use shadeswap_shared::fadroma::QuerierResult;
use shadeswap_shared::fadroma::QueryRequest;
use shadeswap_shared::fadroma::QueryResult;
use crate::state::amm_pair_storage::{{store_trade_history, load_trade_history, load_trade_counter}};
use crate::state::tradehistory::{{ DirectionType}};  
use serde::Deserialize;
use serde::Serialize;
use shadeswap_shared::fadroma::BalanceResponse;
use crate::help_math::calculate_and_print_price;
use shadeswap_shared::custom_fee::{Fee, CustomFee};
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, MessageInfo, ContractInfo, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse,  Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
            BankQuery, WasmQuery,  
            secret_toolkit::snip20,  BlockInfo
        },
        scrt_uint256::Uint256,
        scrt_callback::Callback,
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt_vk::ViewingKey,
    },
 
};

use shadeswap_shared::{
    fadroma::{
        scrt::{
            testing::{mock_dependencies, mock_env,MockApi, MockStorage, MOCK_CONTRACT_ADDR},            
        },
    }
};
use composable_snip20::msg::{{InitMsg as Snip20ComposableMsg, InitConfig as Snip20ComposableConfig}};

pub const FACTORY_CONTRACT_ADDRESS: &str = "FACTORY_CONTRACT_ADDRESS";
pub const CUSTOM_TOKEN_1: &str = "CUSTOM_TOKEN_1";
pub const CUSTOM_TOKEN_2: &str = "CUSTOM_TOKEN_2";
pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";

pub fn mock_deps_with_expected_return_value() -> Extern<MockStorage, MockApi, MockQuerierExpectedValue> {
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(123),
        querier: MockQuerierExpectedValue { portion: 2500 },
    }
}


pub fn mk_amm_settings_a() -> AMMSettings<HumanAddr>{
    AMMSettings{
        lp_fee: Fee{
            nom: 2,
            denom: 100
        },
        shade_dao_fee: Fee {
            nom: 1,
            denom: 100
        },
        shade_dao_address: ContractLink{
            code_hash: "CODEHAS".to_string(),
            address: HumanAddr("TEST".to_string())
        }
    }
}

pub struct MockQuerierExpectedValue{
    portion: u128,
}

impl Querier for MockQuerierExpectedValue{
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
        match &request {
            QueryRequest::Wasm(msg) => {
                match msg {
                    WasmQuery::Smart { contract_addr, .. } => {
                        println!("Factory Address :: {}", contract_addr);
                        match contract_addr.as_str() {                          
                            CUSTOM_TOKEN_2 => {                                
                                QuerierResult::Ok(to_binary(&IntBalanceResponse {
                                    balance: Balance {
                                        amount: Uint128(100000),
                                    },
                                }))
                            },
                            CUSTOM_TOKEN_1 =>{
                                QuerierResult::Ok(to_binary(&IntBalanceResponse {
                                    balance: Balance{
                                        amount: Uint128(10000),
                                    },
                                }))
                            },    
                            _ => unimplemented!()
                        }
                    },                  
                    _ => unimplemented!(),
                }
            },      
            QueryRequest::Bank(msg) => {
                match msg {
                    BankQuery::Balance {address, .. } => {
                        println!("Factory Address :: {}", address);
                        match address.as_str() {
                            CONTRACT_ADDRESS => {
                                QuerierResult::Ok(to_binary(&BalanceResponse{
                                    amount: Coin{
                                        denom: "uscrt".into(),
                                        amount: Uint128(1000000u128),
                                    }
                                }))
                            }, 
                            "cosmos2contract" => {
                                QuerierResult::Ok(to_binary(&BalanceResponse{
                                    amount: Coin{
                                        denom: "uscrt".into(),
                                        amount: Uint128(1000000u128),
                                    }
                                }))
                            },                          
                            _ => {                            
                                unimplemented!()
                            } 
                        }
                    },
                    _ => unimplemented!(),
                }
            },  
            _ => unimplemented!(),
        }
    }

    fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
        self.custom_query(request)
    }

    fn custom_query<T: serde::Serialize, U: DeserializeOwned>(
        &self,
        request: &QueryRequest<T>,
    ) -> StdResult<U> {
        let raw = match shadeswap_shared::fadroma::to_vec(request) {
            Ok(raw) => raw,
            Err(e) => {
                return Err(StdError::generic_err(format!(
                    "Serializing QueryRequest: {}",
                    e
                )))
            }
        };
        match self.raw_query(&raw) {
            Err(sys) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                sys
            ))),
            Ok(Err(err)) => Err(err),
            // in theory we would process the response, but here it is the same type, so just pass through
            Ok(Ok(res)) => from_binary(&res),
        }
    }

    // fn query_balance<U: Into<HumanAddr>>(&self, address: U, denom: &str) -> StdResult<Coin> {
    //     let request = shadeswap_shared::fadroma::BankQuery::Balance {
    //         address: address.into(),
    //         denom: denom.to_string(),
    //     }
    //     .into();
    //     let res: shadeswap_shared::fadroma::BalanceResponse = self.query(&request)?;
    //     Ok(res.amount)
    // }

    // fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
    //     let request = shadeswap_shared::fadroma::BankQuery::AllBalances {
    //         address: address.into(),
    //     }
    //     .into();
    //     let res: shadeswap_shared::fadroma::AllBalanceResponse = self.query(&request)?;
    //     Ok(res.amount)
    // }
}

#[derive(Serialize, Deserialize)]
pub struct IntBalanceResponse {
    pub balance: Balance,
}

pub fn mk_custom_token_amount_test_calculation_price_fee(amount: Uint128, token_pair: TokenPair<HumanAddr>) -> TokenAmount<HumanAddr>{    
    let token = TokenAmount{
        token: token_pair.0.clone(),
        amount: amount.clone(),
    };
    token
}

pub fn mock_config_test_calculation_price_fee(env: Env) -> StdResult<Config<HumanAddr>>
{    
    let seed = to_binary(&"SEED".to_string())?;
    let entropy = to_binary(&"ENTROPY".to_string())?;

    Ok(Config {       
        factory_info: mock_contract_link(FACTORY_CONTRACT_ADDRESS.to_string()),
        lp_token_info: mock_contract_link("LPTOKEN".to_string()),
        pair:      mk_token_pair_test_calculation_price_fee(),
        contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
        viewing_key:  create_viewing_key(&env, seed.clone(), entropy.clone()),
        custom_fee: None
    })
}

pub fn make_init_config_test_calculate_price_fee<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    token_pair: TokenPair<HumanAddr>,
    custom_fee: Option<CustomFee>,
) 
-> StdResult<Config<HumanAddr>> {    
    let seed = to_binary(&"SEED".to_string())?;
    let entropy = to_binary(&"ENTROPY".to_string())?;
    let env = mkenv(CONTRACT_ADDRESS);  
    let msg = InitMsg {
        pair: token_pair.clone(),
        lp_token_contract: ContractInstantiationInfo{
              code_hash: "CODE_HASH".to_string(),
              id :0
        },
        factory_info: ContractLink {
            address: HumanAddr(String::from(FACTORY_CONTRACT_ADDRESS)),
            code_hash: "TEST".to_string()
        },
        prng_seed: seed.clone(),
        entropy: entropy.clone(),
        admin: Some(HumanAddr::from(env.message.sender.clone())),
        callback: Some(Callback {
            contract: ContractLink {
                address: HumanAddr(String::from("CALLBACKADDR")),
                code_hash: "Test".to_string()
            },
            msg: to_binary(&String::from("Welcome bytes"))?,
        }),
        staking_contract: None,
        custom_fee: custom_fee,
    };         
    assert!(init(deps, env.clone(), msg).is_ok());
    let config = load_config(deps)?;
    Ok(config)
}

pub fn mkenv(sender: impl Into<HumanAddr>) -> Env {
    mock_env(sender, &[])
}

pub fn mk_token_pair_test_calculation_price_fee() -> TokenPair<HumanAddr>{
    let pair =  TokenPair(
        TokenType::CustomToken {
            contract_addr: HumanAddr(CUSTOM_TOKEN_1.to_string().clone()),
            token_code_hash: CUSTOM_TOKEN_1.to_string()
        },            
        TokenType::CustomToken {
            contract_addr: HumanAddr(CUSTOM_TOKEN_2.to_string().clone()),
            token_code_hash: CUSTOM_TOKEN_2.to_string()
        }
    );
    pair
}

pub fn mock_contract_link(address: String)-> ContractLink<HumanAddr>{
    ContractLink{
        address: HumanAddr::from(address.clone()),
        code_hash: "CODEHASH".to_string()
    }
}

fn mock_contract_info(address: &str) -> ContractInfo{
    ContractInfo{
        address :HumanAddr::from(address.clone())
    }
}
