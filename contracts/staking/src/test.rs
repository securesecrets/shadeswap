use serde::de::DeserializeOwned;
use shadeswap_shared::msg::staking::InvokeMsg;

#[cfg(test)]
pub mod tests {
    use super::*;
    use cosmwasm_std::{BankQuery, AllBalanceResponse, to_vec, Coin, StdResult, HumanAddr, BalanceResponse, from_binary, StdError, QueryRequest, Empty, Uint128, to_binary, QuerierResult, from_slice, Querier, testing::{MockApi, MockStorage}, Extern, ContractInfo, MessageInfo, BlockInfo, Env, Api, Storage, WasmQuery};
    use secret_toolkit::snip20::Balance;
    use shadeswap_shared::{msg::staking::{{InitMsg,QueryMsg,QueryResponse,  HandleMsg}}, core::ContractLink};
    use crate::state::{{Config , store_config, load_stakers, get_total_staking_amount, load_claim_reward_timestamp,
        load_config, is_address_already_staker, load_claim_reward_info,
        load_staker_info}};    
    use crate::contract::{{init, get_current_timestamp,claim_rewards_for_all_stakers, query, handle, get_staking_percentage}};
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
    use crate::contract::{{get_claim_reward_for_user}};
    use shadeswap_shared::token_type::TokenType;
    use serde::Deserialize;
    use serde::Serialize;
   

    pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";
    pub const LP_TOKEN: &str = "LP_TOKEN";
    pub const REWARD_TOKEN: &str = "REWARD_TOKEN";
    pub const STAKING_CONTRACT_ADDRESS: &str = "STAKING_CONTRACT_ADDRESS";
    
    #[test]
    fn assert_init_config() -> StdResult<()> {   
        let mut deps = mock_deps();  
        let env = mock_env(CONTRACT_ADDRESS,1571797523, 1524,CONTRACT_ADDRESS, &[]);
        let config: Config = make_init_config(&mut deps, env, Uint128(100u128))?;        
        assert_eq!(config.daily_reward_amount, Uint128(100u128));
        assert_eq!(config.reward_token, TokenType::CustomToken{
            contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
            token_code_hash: CONTRACT_ADDRESS.to_string(),
        });
        Ok(())
    }

    #[test]
    fn assert_stake_existing_staker() -> StdResult<()>{
        let mut deps = mock_deps();  
        let env = mock_env("LPTOKEN".to_string(),1656480000, 1524,CONTRACT_ADDRESS,  &[]);    
        let staker = env.message.sender.clone();     
        let mut config: Config = make_init_config(&mut deps, env.clone(), Uint128(10000000000u128))?;     
        let lp_token = ContractLink{
            address: HumanAddr::from("LPTOKEN".to_string()),
            code_hash: "CODE_HASH".to_string(),
        };
        config.lp_token = lp_token.clone();
        store_config(&mut deps, &config)?;
        let receive_msg = HandleMsg::Receive { 
            from: staker.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };

        let result = handle(
            &mut deps,
            env.clone(),  
            receive_msg.clone()          
        )
        .unwrap();

        let is_user_staker = is_address_already_staker(&deps, staker.clone())?;
        let stake_info = load_staker_info(&deps, staker.clone())?;
        assert_eq!(is_user_staker, true);
        assert_eq!(stake_info.amount, Uint128(100u128));

        let result = handle(
            &mut deps,
            env.clone(),
            receive_msg.clone()
        )
        .unwrap();
        let total_amount = get_total_staking_amount(&mut deps)?;
        assert_eq!(total_amount, Uint128(200u128));
        Ok(())
    }

    #[test]
    fn assert_unstake_existing_staker() -> StdResult<()>{
        let mut deps = mock_deps();  
        let env = mock_env("LPTOKEN".to_string(), 1571797523, 1524,CONTRACT_ADDRESS, &[]);
        let staker = env.message.sender.clone();     
        let mut config: Config = make_init_config(&mut deps, env.clone(), Uint128(100u128))?;     
        let lp_token = ContractLink{
            address: HumanAddr::from("LPTOKEN".to_string()),
            code_hash: "CODE_HASH".to_string(),
        };
        config.lp_token = lp_token.clone();        
        store_config(&mut deps, &config)?;
        let receive_msg = HandleMsg::Receive { 
            from: staker.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };

        let result = handle(
            &mut deps,
            env.clone(),
            receive_msg.clone()
        )
        .unwrap();            
        let stake_info = load_staker_info(&deps, staker.clone())?;     
        assert_eq!(stake_info.amount, Uint128(100u128));
        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Unstake {amount: Uint128(100u128), remove_liqudity: Some(false)},
        )
        .unwrap();
        let stake_info = load_staker_info(&deps, staker.clone())?;    
        let claim_reward_inf = load_claim_reward_info(&deps, staker.clone()) ?;
        assert_eq!(stake_info.amount, Uint128(0u128));
        Ok(())
    }


    #[test]
    fn assert_claim_rewards() -> StdResult<()>{
        let staker_a = HumanAddr("STAKERA".to_string());
        let staker_b = HumanAddr("STAKERB".to_string());  
        let mut deps = mock_deps();  
        let current_timestamp = get_current_timestamp()?;
        let env_a = mock_env("LPTOKEN".to_string(), current_timestamp.u128() as u64, 1524, CONTRACT_ADDRESS,  &[]);
        let mut config: Config = make_init_config(&mut deps, env_a.clone(), Uint128(1000000000000u128))?;           
        let lp_token = ContractLink{
            address: HumanAddr::from("LPTOKEN".to_string()),
            code_hash: "CODE_HASH".to_string(),
        };
        config.lp_token = lp_token.clone();            
        store_config(&mut deps, &config)?;
        let receive_msg = HandleMsg::Receive { 
            from: staker_a.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker_a.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };
        let result = handle(
            &mut deps,
            env_a.clone(),          
            receive_msg.clone()            
        )
        .unwrap();
        let is_user_staker = is_address_already_staker(&deps, staker_a.clone())?;        
        assert_eq!(is_user_staker, true);
        let env_b = mock_env("LPTOKEN".to_string(), (current_timestamp + Uint128(100u128)).u128() as u64, 1527, CONTRACT_ADDRESS, &[]);
        let receive_msg = HandleMsg::Receive { 
            from: staker_b.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker_b.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };
        let result = handle(
            &mut deps,
            env_b.clone(),        
            receive_msg.clone()
        )
        .unwrap();            
        let current_time = current_timestamp + Uint128(1000u128);              
        claim_rewards_for_all_stakers(&mut deps, current_time)?;
        let claim_reward_info_a = load_claim_reward_info(&deps,staker_a.clone())?;
        assert_eq!(claim_reward_info_a.amount, Uint128(578703703));      
        let claim_reward_info_b = load_claim_reward_info(&deps,staker_b.clone())?;
        assert_eq!(claim_reward_info_b.amount, Uint128(0));       
        Ok(())
    }

      
    #[test]
    fn assert_get_staking_percentage_success() -> StdResult<()>{
        let mut deps = mock_deps();  
        let mut env_a = mock_env("LPTOKEN".to_string(), 14525698, 1425,STAKING_CONTRACT_ADDRESS, &[]);
        let mut env_b = mock_env("LPTOKEN".to_string(), 14525710, 1435,STAKING_CONTRACT_ADDRESS, &[]);
        let mut config: Config = make_init_config(&mut deps, env_a.clone(), Uint128(100u128))?;   
        let staker_a = HumanAddr("STAKERA".to_string());
        let staker_b = HumanAddr("STAKERB".to_string()); 
        let lp_token = ContractLink{
            address: HumanAddr::from("LPTOKEN".to_string()),
            code_hash: "CODE_HASH".to_string(),
        };
        config.lp_token = lp_token.clone();            
        store_config(&mut deps, &config)?;      
        let receive_msg = HandleMsg::Receive { 
            from: staker_a.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker_a.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };
        let result = handle(
            &mut deps,
            env_a.clone(),        
            receive_msg.clone()
        )
        .unwrap();
        let is_user_staker = is_address_already_staker(&deps, staker_a.clone())?;
        let stake_info = load_staker_info(&deps, staker_a.clone())?;
        assert_eq!(is_user_staker, true);
        let receive_msg = HandleMsg::Receive { 
            from: staker_a.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker_b.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };
        let result = handle(
            &mut deps,
            env_b.clone(),      
            receive_msg.clone()
        )
        .unwrap();
        let staking_percentage_a = get_staking_percentage(&mut deps, staker_a.clone(), Uint128(100u128))?;
        println!("{}", staking_percentage_a);
        assert_eq!(staking_percentage_a, Uint128(50u128));
        let staking_percentage_b = get_staking_percentage(&mut deps, staker_b.clone(), Uint128(100u128))?;
        println!("{}", staking_percentage_b);
        assert_eq!(staking_percentage_b, Uint128(50u128));
        Ok(())
    }

    #[test]
    fn assert_set_and_check_viewing_key_for_query_success() -> StdResult<()>{
        let mut deps = mock_deps();  
        let staker_a = HumanAddr("STAKERA".to_string());     
        let mut env_a = mock_env("LPTOKEN".to_string(), 14525698, 1425,STAKING_CONTRACT_ADDRESS, &[]);       
        let mut env_b = mock_env( "STAKERA".to_string(),14528698, 1465,STAKING_CONTRACT_ADDRESS, &[]);       
        let mut config: Config = make_init_config(&mut deps, env_a.clone(), Uint128(100u128))?;   
        let staker_a = HumanAddr("STAKERA".to_string());       
        let lp_token = ContractLink{
            address: HumanAddr::from("LPTOKEN".to_string()),
            code_hash: "CODE_HASH".to_string(),
        };
        config.lp_token = lp_token.clone();            
        store_config(&mut deps, &config)?;    
        // set staker_a   
        let receive_msg = HandleMsg::Receive { 
            from: staker_a.clone(),
            msg: Some(to_binary(&InvokeMsg::Stake{
                    amount: Uint128(100u128),
                    from: staker_a.clone()
            }).unwrap()),
            amount: Uint128(100u128)
        };
        let result = handle(
            &mut deps,
            env_a.clone(),        
            receive_msg.clone()
        )
        .unwrap();

        let is_user_staker = is_address_already_staker(&deps, staker_a.clone())?;
        let stake_info = load_staker_info(&deps, staker_a.clone())?;
        assert_eq!(is_user_staker, true);
        let set_vk_msg = HandleMsg::SetVKForStaker { key: "password".to_string()};
        let result = handle(
            &mut deps,
            env_b.clone(),      
            set_vk_msg.clone()
        )
        .unwrap();
     
        // get query of staker
        let binary_msg = get_claim_reward_for_user(&deps, HumanAddr::from("STAKERA".to_string()), "password".to_string(), Uint128(14528698000))?;
        let claimable_reward_for_staker: QueryResponse = from_binary(&binary_msg)?;
        if let QueryResponse::ClaimReward { amount} = claimable_reward_for_staker{
            assert_ne!(amount, Uint128(0));
        }       
        
        Ok(())
    }

    
    fn make_init_config<S: Storage, A: Api, Q: Querier>(
        deps: DepsMut, 
        env: Env,
        amount: Uint128) -> StdResult<Config> {    
        let msg = InitMsg {
            staking_amount: amount.clone(),         
            reward_token: TokenType::CustomToken{
                contract_addr: HumanAddr::from(CONTRACT_ADDRESS),
                token_code_hash: CONTRACT_ADDRESS.to_string(),
            },           
            pair_contract: ContractLink {
                address: HumanAddr::from(CONTRACT_ADDRESS),
                code_hash: "".to_string().clone(),
            },
            prng_seed: to_binary(&"prng")?
        };         
        assert!(init(deps, env.clone(), msg).is_ok());
        let config = load_config(deps)?;
        Ok(config)
    }

    pub fn mock_env<U: Into<HumanAddr>>(sender: U, time: u64, height: u64, contract_address: &str, sent: &[Coin]) -> Env {
        Env {
            block: BlockInfo {
                height: height,
                time: time,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: sender.into(),
                sent_funds: sent.to_vec(),
            },
            contract: ContractInfo {
                address: HumanAddr::from(contract_address),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        }
    }

    fn mock_deps() -> Deps<MockStorage, MockApi, MockQuerier> {
        Extern {
            storage: MockStorage::default(),
            api: MockApi::new(123),
            querier: MockQuerier { portion: 2500 },
        }
    }

    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
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
                                CONTRACT_ADDRESS => {
                                    QuerierResult::Ok(to_binary(&BalanceResponse{
                                        amount: Coin{
                                            denom: "uscrt".into(),
                                            amount: Uint128(1000000u128),
                                        }
                                    }))
                                },
                                REWARD_TOKEN => {
                                    QuerierResult::Ok(to_binary(&IntBalanceResponse {
                                        balance: Balance {
                                            amount: Uint128(1000000u128),
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
            let raw = match to_vec(request) {
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
            let request = BankQuery::Balance {
                address: address.into(),
                denom: denom.to_string(),
            }
            .into();
            let res: BalanceResponse = self.query(&request)?;
            Ok(res.amount)
        }
    
        fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
            let request = BankQuery::AllBalances {
                address: address.into(),
            }
            .into();
            let res: AllBalanceResponse = self.query(&request)?;
            Ok(res.amount)
        }
    
       
    }

}




