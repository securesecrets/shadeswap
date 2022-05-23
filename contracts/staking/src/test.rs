
use shadeswap_shared::fadroma::secret_toolkit::snip20::Balance;
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
use serde::de::DeserializeOwned;
use shadeswap_shared::fadroma::Empty;
use shadeswap_shared::fadroma::from_slice;
use shadeswap_shared::fadroma::QuerierResult;
use shadeswap_shared::fadroma::QueryRequest;
use shadeswap_shared::fadroma::QueryResult;
use shadeswap_shared::fadroma::BalanceResponse;

#[cfg(test)]
pub mod tests {
    use super::*;
    use shadeswap_shared::msg::staking::{{InitMsg,QueryMsg, InvokeMsg, HandleMsg}};
    use crate::state::{{Config , store_config, get_total_staking_amount, 
        load_config, is_address_already_staker,
        load_staker_info}};
    use crate::contract::{{init, handle}};
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
   
    use shadeswap_shared::token_type::TokenType;
    use serde::Deserialize;
    use serde::Serialize;
   

    pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";
    pub const LP_TOKEN: &str = "LP_TOKEN";
    pub const REWARD_TOKEN: &str = "REWARD_TOKEN";
    
    #[test]
    fn assert_init_config() -> StdResult<()> {   
        let mut deps = mock_deps();  
        let env = mock_env(CONTRACT_ADDRESS, &[]);
        let config: Config = make_init_config(&mut deps, env, Uint128(100u128))?;        
        assert_eq!(config.daily_reward_amount, Uint128(100u128));
        assert_eq!(config.reward_token, TokenType::CustomToken{
            contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
            token_code_hash: CONTRACT_ADDRESS.to_string(),
        });
        assert_eq!(config.lp_token, TokenType::CustomToken{
            contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
            token_code_hash: CONTRACT_ADDRESS.to_string(),
        });
        Ok(())
    }

    #[test]
    fn assert_stake_existing_staker() -> StdResult<()>{
        let mut deps = mock_deps();  
        let env = mock_env(CONTRACT_ADDRESS, &[]);
        let staker = env.message.sender.clone();     
        let config: Config = make_init_config(&mut deps, env.clone(), Uint128(100u128))?;     
        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Receive {
                msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker.clone()
                })?),
                
            },
        )
        .unwrap();
        let is_user_staker = is_address_already_staker(&deps, staker.clone())?;
        let stake_info = load_staker_info(&deps, staker)?;
        assert_eq!(is_user_staker, true);
        assert_eq!(stake_info.amount, Uint128(100u128));
        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Receive {
                msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker.clone()
                })?),
                
            },
        )
        .unwrap();
        let total_amount = get_total_staking_amount(&mut deps)?;
        assert_eq!(total_amount, Uint128(200u128));
        Ok(())
    }

    #[test]
    fn assert_unstake_existing_staker() -> StdResult<()>{
        let mut deps = mock_deps();  
        let env = mock_env(CONTRACT_ADDRESS, &[]);
        let staker = env.message.sender.clone();     
        let config: Config = make_init_config(&mut deps, env.clone(), Uint128(100u128))?;     
        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Receive {
                msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker.clone()
                })?),
                
            },
        )
        .unwrap();            
        let stake_info = load_staker_info(&deps, staker.clone())?;     
        assert_eq!(stake_info.amount, Uint128(100u128));
        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Unstake {},
        )
        .unwrap();
        let stake_info = load_staker_info(&deps, staker.clone())?;     
        assert_eq!(stake_info.amount, Uint128(0u128));
        Ok(())
    }

    
    fn make_init_config<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>, 
        env: Env,
        amount: Uint128) -> StdResult<Config> {    
        let msg = InitMsg {
            staking_amount: amount.clone(),
            lp_token: TokenType::CustomToken{
                contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
                token_code_hash: CONTRACT_ADDRESS.to_string(),
            },
            reward_token: TokenType::CustomToken{
                contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
                token_code_hash: CONTRACT_ADDRESS.to_string(),
            }
        };         
        assert!(init(deps, env.clone(), msg).is_ok());
        let config = load_config(deps)?;
        Ok(config)
    }


    fn mock_deps() -> Extern<MockStorage, MockApi, MockQuerier> {
        Extern {
            storage: MockStorage::default(),
            api: MockApi::new(123),
            querier: MockQuerier { portion: 2500 },
        }
    }
}

struct MockQuerier{
    portion: u128,
}

impl Querier for MockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
        match &request {
            QueryRequest::Wasm(msg) => {
                match msg {
                    WasmQuery::Smart { contract_addr, .. } => {
                        println!("Factory Address :: {}", contract_addr);
                        match contract_addr.as_str() {                    
                            // CONTRACT_ADDRESS => {
                            //     QuerierResult::Ok(to_binary(&BalanceResponse{
                            //         amount: Coin{
                            //             denom: "uscrt".into(),
                            //             amount: Uint128(1000000u128),
                            //         }
                            //     }))
                            // }
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

    fn query_balance<U: Into<HumanAddr>>(&self, address: U, denom: &str) -> StdResult<Coin> {
        let request = shadeswap_shared::fadroma::BankQuery::Balance {
            address: address.into(),
            denom: denom.to_string(),
        }
        .into();
        let res: shadeswap_shared::fadroma::BalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
        let request = shadeswap_shared::fadroma::BankQuery::AllBalances {
            address: address.into(),
        }
        .into();
        let res: shadeswap_shared::fadroma::AllBalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }
}

