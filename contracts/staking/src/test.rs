


pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";
pub const LP_TOKEN: &str = "LP_TOKEN";
pub const REWARD_TOKEN: &str = "REWARD_TOKEN";
pub const STAKING_CONTRACT_ADDRESS: &str = "STAKING_CONTRACT_ADDRESS";

#[cfg(test)]
pub mod tests {
    use super::*;
    use cosmwasm_std::{StdResult, Uint128, testing::{mock_info}, Addr, Decimal};
    use shadeswap_shared::{core::{TokenType}};    
    
    use crate::{test::test_help_lib::{mock_custom_env, make_init_config, mock_dependencies, mock_custom_env_b}, state::{Config, claim_reward_info_r, ClaimRewardsInfo}, operations::{calculate_staker_shares, stake, get_total_stakers_count, claim_rewards_for_all_stakers, calculate_staking_reward}};
    
     
    
    #[test]
    fn assert_init_config() -> StdResult<()> {   
        let mut deps = mock_dependencies(&[]);  
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let config: Config = make_init_config(deps.as_mut(), env, Uint128::from(100u128))?;        
        assert_eq!(config.daily_reward_amount, Uint128::from(100u128));
        assert_eq!(config.reward_token, TokenType::CustomToken{
            contract_addr: Addr::unchecked(CONTRACT_ADDRESS),
            token_code_hash: CONTRACT_ADDRESS.to_string(),
        });
        Ok(())
    }

    #[test]
    fn assert_calculate_user_share_first_return_100() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let _config: Config = make_init_config(deps.as_mut(), env, Uint128::from(100u128))?;       
        let user_shares = calculate_staker_shares(deps.as_mut().storage, Uint128::from(100u128))?;
        assert_eq!(user_shares, Decimal::zero());
        Ok(())
    }

    // total = 1500
    // amount = 500
    // share = 500/1500 = 0.33333333333333333
    #[test]
    fn assert_calculate_user_share_already_return_() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(100u128))?;
        let _stake = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1500u128),Addr::unchecked("Staker"))?;       
        let user_shares = calculate_staker_shares(deps.as_mut().storage, Uint128::from(500u128))?;
        assert_eq!(user_shares, Decimal::from_atomics(Uint128::new(333333333333333333), 18).unwrap());
        Ok(())
    }

    #[test]
    fn assert_get_total_stakers_count_return_3() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let _env_b = mock_custom_env(CONTRACT_ADDRESS,1571797854, 1534);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(100u128))?;
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),Addr::unchecked("StakerA"))?;    
        let _stake_b = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1500u128),Addr::unchecked("StakerB"))?;       
        let _stake_c = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1700u128),Addr::unchecked("StakerC"))?;   
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),Addr::unchecked("StakerA"))?;           
        let total_stakers_count = get_total_stakers_count(deps.as_mut().storage);
        assert_eq!(total_stakers_count, Uint128::from(3u128));
        Ok(())
    }

    #[test]
    fn assert_staking_with_claim_rewards() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,15000000, 1500);
        let env_b = mock_custom_env(CONTRACT_ADDRESS,16000000, 1534);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),Addr::unchecked("StakerA"))?;    
        let _stake_b = stake(deps.as_mut(), env_b.clone(),mock_info.clone(), Uint128::from(1500u128),Addr::unchecked("StakerB"))?;    
        claim_rewards_for_all_stakers(deps.as_mut().storage,Uint128::from(16000000u128))?;
        let claim_reward_info_a: ClaimRewardsInfo  = claim_reward_info_r(deps.as_mut().storage).load(Addr::unchecked("StakerA").as_bytes())?;
        let claim_reward_info_b: ClaimRewardsInfo  = claim_reward_info_r(deps.as_mut().storage).load(Addr::unchecked("StakerB").as_bytes())?;
        assert_eq!(claim_reward_info_a.amount, Uint128::from(22222u128));
        assert_eq!(claim_reward_info_b.amount, Uint128::from(33333u128));
        assert_eq!(claim_reward_info_a.last_time_claimed, Uint128::from(16000000u128));
        assert_eq!(claim_reward_info_b.last_time_claimed, Uint128::from(16000000u128));
        Ok(())
    }

    // const reward_amount = 300000
    // ration is 0.4 : 0.6 (Staker_A : Staker_B)
    // seconds 86,400,000
    // stake -> staker_a 15000000time -> 1000amount
    // stake -> staker_b 16000000time -> 1500amount
    // 1. (300000 * 10000000 / 86400000) * 0.4 = 1388-> Staker A
    // 
    #[test]
    fn assert_calculate_staking_reward() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,15000000, 1500);
        let env_b = mock_custom_env(CONTRACT_ADDRESS,16000000, 1534);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),Addr::unchecked("StakerA"))?;    
        let _stake_b = stake(deps.as_mut(), env_b.clone(),mock_info.clone(), Uint128::from(1500u128),Addr::unchecked("StakerB"))?;   
        let last_timestamp = Uint128::from(15000000u128);
        let current_timestamp  = Uint128::from(16000000u128);
        let staking_reward = calculate_staking_reward(deps.as_mut().storage, Uint128::from(1000u128),last_timestamp, current_timestamp)?;
        assert_eq!(staking_reward, Uint128::from(1388u128));       
        Ok(())
    }

    #[test]
    fn assert_staking_first_time_store_timestamp() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env_b(CONTRACT_ADDRESS,15000000, 1500);        
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),Addr::unchecked("StakerA"))?;    
        let claim_reward_info_b: ClaimRewardsInfo  = claim_reward_info_r(deps.as_mut().storage).load(Addr::unchecked("StakerA").as_bytes())?;
        assert_eq!(claim_reward_info_b.amount, Uint128::zero());
        assert_eq!(claim_reward_info_b.last_time_claimed, Uint128::from(15000000u128));       
        Ok(())
    }
}

#[cfg(test)]
pub mod test_help_lib{
    use super::*;
    use cosmwasm_std::{Uint128, DepsMut, Env, StdResult, Addr, testing::{mock_info, MockStorage, MockApi}, BlockInfo, TransactionInfo, ContractInfo, Timestamp, to_binary, OwnedDeps, Coin, Querier, QuerierResult, BalanceResponse, from_slice, Empty, QueryRequest};
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{staking::InitMsg, core::{TokenType, ContractLink}, snip20::{QueryAnswer, manager::Balance}};

    use crate::{state::{Config, config_r, config_w}, contract::instantiate};

    pub fn make_init_config(
            mut deps: DepsMut, 
            env: Env,
            amount: Uint128
        ) -> StdResult<Config> 
    {    
        let info = mock_info("Sender", &[]);
        let msg = InitMsg {
            staking_amount: amount.clone(),         
            reward_token: TokenType::CustomToken{
                contract_addr: Addr::unchecked(CONTRACT_ADDRESS),
                token_code_hash: CONTRACT_ADDRESS.to_string(),
            },           
            pair_contract: ContractLink {
                address: Addr::unchecked(CONTRACT_ADDRESS),
                code_hash: "".to_string().clone(),
            },
            prng_seed: to_binary(&"prng")?,
            lp_token: ContractLink { address: Addr::unchecked("".to_string()), code_hash: "".to_string() },
            authenticator: None,
            admin: Addr::unchecked("Sender"),
        };         
        assert!(instantiate(deps.branch(), env.clone(),info.clone(), msg).is_ok());
        let mut config = config_r(deps.storage).load()?;
        config.lp_token = ContractLink{ address: Addr::unchecked(LP_TOKEN), code_hash: "".to_string() };
        config_w(deps.storage).save(&config)?;
        Ok(config)
    }
  
    pub fn mock_custom_env(address: &str, height: u64, time: u64) -> Env {
        Env {
            block: BlockInfo {
                height: height,
                time: Timestamp::from_nanos(time),
                chain_id: "pulsar-2".to_string(),
            },
            transaction: Some(TransactionInfo { index: 3 }),
            contract: ContractInfo {
                address: Addr::unchecked(address),
                code_hash: "".to_string(),
            },
        }
    }

    pub fn mock_custom_env_b(address: &str, height: u64, _time: u64) -> Env {
        Env {
            block: BlockInfo {
                height: height,
                time: Timestamp::from_seconds(15000),
                chain_id: "pulsar-2".to_string(),
            },
            transaction: Some(TransactionInfo { index: 3 }),
            contract: ContractInfo {
                address: Addr::unchecked(address),
                code_hash: "".to_string(),
            },
        }
    }

    pub fn mock_dependencies(
        _contract_balance: &[Coin],
      ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let _contract_addr = Addr::unchecked(CONTRACT_ADDRESS);
        OwnedDeps {
          storage: MockStorage::default(),
          api: MockApi::default(),
          querier: MockQuerier{portion :100},
            custom_query_type: std::marker::PhantomData,      
        }
      }

    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }
      
    pub struct MockQuerier{
        portion: u128,
    }
    
    impl Querier for MockQuerier {
        fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Bank(msg) => {
                    match msg {
                        cosmwasm_std::BankQuery::Balance { address, denom: _ } => {
                            match address.as_str() {
                                _CUSTOM_TOKEN_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(1000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _FACTORY_CONTRACT_ADDRESS => {
                                    let balance = to_binary(&BalanceResponse{
                                        amount: Coin{
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        }
                                    }).unwrap();                                  
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _CUSTOM_TOKEN_1 => {
                                    let balance = to_binary(&BalanceResponse{
                                        amount: Coin{
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        }
                                    }).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _ => {
                                    let response : &str= &address.to_string();
                                    println!("{}", response);
                                    unimplemented!("wrong tt address")   
                                }                      

                            }
                        },
                        cosmwasm_std::BankQuery::AllBalances { address: _ } => todo!(),
                        _ => todo!(),
                    }
                },
                QueryRequest::Custom(_) => todo!(),
                QueryRequest::Wasm(msg) =>{ 
                    match msg {
                        cosmwasm_std::WasmQuery::Smart { contract_addr, code_hash: _, msg: _ } => {
                            match contract_addr.as_str(){                              
                                _CUSTOM_TOKEN_1 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(10000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _CUSTOM_TOKEN_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(10000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _ => {
                                    let response : &str= &contract_addr.to_string();
                                    println!("{}", response);
                                    unimplemented!("wrong address")
                                },
                            }
                        },
                        cosmwasm_std::WasmQuery::ContractInfo { contract_addr: _ } => todo!(),
                        cosmwasm_std::WasmQuery::Raw { key: _, contract_addr: _ } => todo!(),
                        _ => todo!(),
                    }
                },
                _ => todo!(),
            }
        }
    }
}
   


