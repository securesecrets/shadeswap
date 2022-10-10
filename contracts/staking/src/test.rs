use serde::de::DeserializeOwned;
use shadeswap_shared::msg::staking::InvokeMsg;

pub const CONTRACT_ADDRESS: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const LP_TOKEN: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const STAKER_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const STAKER_B: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";
pub const STAKER_C:& str = "secret1nulgwu6es24us9urgyvms7y02txyg0s02msgzw";
pub const SENDER:& str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";

#[cfg(test)]
pub mod tests {
    use super::*;
    use cosmwasm_std::{BankQuery, AllBalanceResponse, to_vec, Coin, StdResult, BalanceResponse, from_binary, StdError, QueryRequest, Empty, Uint128, to_binary, QuerierResult, from_slice, Querier, testing::{MockApi, MockStorage, mock_info}, MessageInfo, BlockInfo, Env, Api, Storage, WasmQuery, Addr, Decimal};
    use secret_multi_test::Contract;
    use shadeswap_shared::{msg::staking::{{InitMsg,QueryMsg,QueryResponse,  ExecuteMsg}}, core::{ContractLink, TokenType}, c_std::{Deps, OwnedDeps, CustomQuery}};    
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
    use crate::{test::test_help_lib::{mock_custom_env, make_init_config, mock_dependencies, MockQuerier}, state::{Config, claim_reward_info_r, ClaimRewardsInfo, last_reward_time_r, stakers_r, reward_token_list_r, reward_token_r, RewardTokenInfo, staker_index_w, staker_index_r, total_staked_r}, operations::{calculate_staker_shares, stake, get_total_stakers_count, claim_rewards_for_all_stakers, calculate_staking_reward, unstake}, contract::instantiate};
 
    #[test]
    fn assert_init_config() -> StdResult<()> {   
        let mut deps = mock_dependencies(&[]);  
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let config: Config = make_init_config(deps.as_mut(), env, Uint128::from(100u128))?;        
        assert_eq!(config.daily_reward_amount, Uint128::from(100u128));
        assert_eq!(config.reward_token, TokenType::CustomToken{
            contract_addr: deps.as_mut().api.addr_validate(CONTRACT_ADDRESS)?,
            token_code_hash: CONTRACT_ADDRESS.to_string(),
        });
        Ok(())
    }

    #[test]
    fn assert_calculate_user_share_first_return_100() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let _config: Config = make_init_config(deps.as_mut(), env, Uint128::from(100u128))?;       
        let user_shares = calculate_staker_shares(deps.as_mut().storage, Uint128::from(100u128)).unwrap();
        assert_eq!(user_shares, Decimal::one());
        Ok(())
    }

    // total = 1500
    // amount = 500
    // share = 500/1500 = 0.33333333333333333
    #[test]
    fn assert_calculate_user_share_already_return_() -> StdResult<()>{
        let mut deps: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]).into();
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]).into();
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(100u128))?;
        let _stake = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1500u128),deps_owned.as_mut().api.addr_validate(STAKER_A)?)?;       
        let user_shares = calculate_staker_shares(deps.as_mut().storage, Uint128::from(500u128)).unwrap();
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
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A)?)?;    
        let _stake_b = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1500u128),deps_owned.as_mut().api.addr_validate(STAKER_B)?)?;       
        let _stake_c = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1700u128),deps_owned.as_mut().api.addr_validate(STAKER_C)?)?;   
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A)?)?;           
        let total_stakers_count = get_total_stakers_count(deps.as_mut().storage);
        assert_eq!(total_stakers_count, Uint128::from(3u128));
        Ok(())
    }

    #[test]
    fn assert_unstake_set_claimable_to_zero() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS,1571797523, 1524);
        let env_b = mock_custom_env(CONTRACT_ADDRESS,1571797854, 1534);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let unstake_mock_info = mock_info(STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(100u128))?;
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(deps.as_mut(), env.clone(),stake_mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A)?)?;    
        let _stake_b = stake(deps.as_mut(), env.clone(),stake_mock_info.clone(), Uint128::from(1500u128),deps_owned.as_mut().api.addr_validate(STAKER_B)?)?;       
        let _stake_c = stake(deps.as_mut(), env.clone(),stake_mock_info.clone(), Uint128::from(1700u128),deps_owned.as_mut().api.addr_validate(STAKER_C)?)?;   
        let _unstake_a = unstake (deps.as_mut(), env.clone(),unstake_mock_info.clone(), Uint128::from(1000u128), Some(true))?;           
        let total_stakers_count = get_total_stakers_count(deps.as_mut().storage);
        let claim_reward_info_a: Vec<ClaimRewardsInfo>  = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        let staker_info = stakers_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes());
        match staker_info {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err.to_string(), "staking::state::StakingInfo not found".to_string()),
        }
        assert_eq!(total_stakers_count, Uint128::from(3u128));
        assert_eq!(claim_reward_info_a[0].amount, Uint128::zero());
        assert_eq!(claim_reward_info_a[0].last_time_claimed, Uint128::new(1524000));
        Ok(())
    }

    #[test]
    fn assert_staking_with_claim_rewards() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 1500000);
        let env_b = mock_custom_env(CONTRACT_ADDRESS, 1534, 1600000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;      
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        // STAKER A
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap()).unwrap();   
        // assert there is no claim reward for staker a
        let claim_reward_info_a = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes());
        // Claimable for staker A throws Exception
        match claim_reward_info_a{
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err, StdError::not_found("alloc::vec::Vec<staking::state::ClaimRewardsInfo>")),
        }

        // Assert Total Staker Count = 1m Total Staker Amount = 1000, Index - 0
        let total_stakers_count = get_total_stakers_count(deps.as_mut().storage);
        assert_eq!(total_stakers_count, Uint128::one());
        let index = staker_index_r(deps.as_mut().storage).load(&Uint128::zero().to_be_bytes()).unwrap();
        assert_eq!(index, deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap());
        let total_staked_amount = total_staked_r(deps.as_mut().storage).load().unwrap();
        assert_eq!(total_staked_amount, Uint128::new(1000u128));
        
        // STAKER B
        let _stake_b = stake(deps.as_mut(), env_b.clone(),mock_info.clone(), Uint128::from(1500u128),deps_owned.as_mut().api.addr_validate(STAKER_B)?)?;    

        // Assert Staker A claimable reward
        let claim_reward_info_a = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes()).unwrap();
        assert_eq!(claim_reward_info_a[0].amount, Uint128::new(347222u128));

        // Claimable for staker B throws Exception
        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_B)?.as_bytes());
         // Claimable for staker A throws Exception
        match claim_reward_info_b{
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err, StdError::not_found("alloc::vec::Vec<staking::state::ClaimRewardsInfo>")),
        }

        let last_claimed_timestamp = last_reward_time_r(deps.as_mut().storage).load().unwrap();
        assert_eq!(last_claimed_timestamp,Uint128::from(1600000000u128));
        // timestamp 1600000000
        claim_rewards_for_all_stakers(deps.as_mut().storage, Uint128::from(1600000000u128)).unwrap();
        let claim_reward_info_a: Vec<ClaimRewardsInfo>  = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        let claim_reward_info_b: Vec<ClaimRewardsInfo>  = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_B)?.as_bytes())?;
        let total_stakers_count = get_total_stakers_count(deps.as_mut().storage);
        let last_timestamp = last_reward_time_r(deps.as_mut().storage).load().unwrap();
        assert_eq!(total_stakers_count, Uint128::new(2u128));
        assert_eq!(last_timestamp, Uint128::new(1600000000u128));
        assert_eq!(claim_reward_info_a[0].amount, Uint128::new(347222u128));
        assert_eq!(claim_reward_info_b[0].amount, Uint128::new(0u128));
        assert_eq!(claim_reward_info_a[0].last_time_claimed, Uint128::new(1600000000u128));
        assert_eq!(claim_reward_info_b[0].last_time_claimed, Uint128::new(1600000000u128));

        // move timestamp 1700000000 
        claim_rewards_for_all_stakers(deps.as_mut().storage, Uint128::from(1700000000u128)).unwrap();
        let claim_reward_info_a: Vec<ClaimRewardsInfo>  = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        let claim_reward_info_b: Vec<ClaimRewardsInfo>  = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_B)?.as_bytes())?;
        assert_eq!(claim_reward_info_a[0].amount, Uint128::new(486110u128));
        assert_eq!(claim_reward_info_b[0].amount, Uint128::new(208333u128));
        assert_eq!(claim_reward_info_a[0].last_time_claimed, Uint128::new(1700000000u128));
        assert_eq!(claim_reward_info_b[0].last_time_claimed, Uint128::new(1700000000u128));
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
        let env = mock_custom_env(LP_TOKEN,1500, 15000000);
        let env_b = mock_custom_env(LP_TOKEN, 1534, 16000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A)?)?;    
        let _stake_b = stake(deps.as_mut(), env_b.clone(),mock_info.clone(), Uint128::from(1500u128),deps_owned.as_ref().api.addr_validate(STAKER_B)?)?;   
        let last_timestamp = Uint128::from(15000000u128);
        let current_timestamp  = Uint128::from(16000000u128);
        let user_shares = calculate_staker_shares(deps.as_mut().storage, Uint128::from(1000u128)).unwrap();
        let reward_tokens = reward_token_list_r(deps.as_mut().storage).load().unwrap();
        let reward_token_info: RewardTokenInfo = reward_token_r(deps.as_mut().storage).load(reward_tokens[0].to_owned().as_bytes()).unwrap();
        let staking_reward = calculate_staking_reward(Uint128::from(1000u128),
            user_shares, last_timestamp, current_timestamp, reward_token_info.amount)?;
        assert_eq!(staking_reward, Uint128::from(1388u128));       
        Ok(())
    }

    #[test]
    fn assert_staking_first_time_store_timestamp() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500,15000000);        
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), env.clone(), Uint128::from(300000u128))?;
        let mut deps_owned:  OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        stake(deps.as_mut(), env.clone(),mock_info.clone(), Uint128::from(1000u128),deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap()).unwrap();    
        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap().as_bytes());
        match claim_reward_info_b{
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err, StdError::not_found("alloc::vec::Vec<staking::state::ClaimRewardsInfo>")),
        }       
        Ok(())
    }
}

#[cfg(test)]
pub mod test_help_lib{
    use super::*;
    use cosmwasm_std::{Uint128, DepsMut, Env, StdResult, Addr, testing::{mock_info, MockStorage, MockApi}, BlockInfo, TransactionInfo, ContractInfo, Timestamp, to_binary, OwnedDeps, Coin, Querier, QuerierResult, BalanceResponse, from_slice, Empty, QueryRequest};
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{staking::InitMsg, core::{TokenType, ContractLink, Fee}, snip20::{QueryAnswer, manager::Balance}};

    use crate::{state::{Config, config_r, config_w}, contract::instantiate};

    pub fn make_init_config(
            mut deps: DepsMut, 
            env: Env,
            amount: Uint128
        ) -> StdResult<Config> 
    {    
        let info = mock_info(SENDER, &[]);
        let msg = InitMsg {
            staking_amount: amount.clone(),         
            reward_token: TokenType::CustomToken{
                contract_addr: deps.api.addr_validate(CONTRACT_ADDRESS)?,
                token_code_hash: CONTRACT_ADDRESS.to_string(),
            },           
            pair_contract: ContractLink {
                address: deps.api.addr_validate(CONTRACT_ADDRESS)?,
                code_hash: "".to_string().clone(),
            },
            prng_seed: to_binary(&"prng")?,
            lp_token: ContractLink { address: Addr::unchecked("".to_string()), code_hash: "".to_string() },
        };         
        assert!(instantiate(deps.branch(), env.clone(),info.clone(), msg).is_ok());
        let mut config = config_r(deps.storage).load()?;
        config.lp_token = ContractLink{ address: deps.api.addr_validate(LP_TOKEN)?, code_hash: "".to_string() };
        config_w(deps.storage).save(&config)?;
        Ok(config)
    }
  
    pub fn mock_custom_env(address: &str, height: u64, time: u64) -> Env {
        Env {
            block: BlockInfo {
                height: height,
                time: Timestamp::from_seconds(time),
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
                        cosmwasm_std::BankQuery::Balance { address, denom:_ } => {
                            match address.as_str() {
                                _custom_token_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(1000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _factory_contract_address => {
                                    let balance = to_binary(&BalanceResponse{
                                        amount: Coin{
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        }
                                    }).unwrap();                                  
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _custom_token_1 => {
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
                                    unimplemented!("{} not implemented", address.as_str()) 
                                }                      

                            }
                        },
                        cosmwasm_std::BankQuery::AllBalances { address:_ } => todo!(),
                        _ => todo!(),
                    }
                },
                QueryRequest::Custom(_) => todo!(),
                QueryRequest::Wasm(msg) =>{ 
                    match msg {
                        cosmwasm_std::WasmQuery::Smart { contract_addr, code_hash:_, msg:_ } => {
                            match contract_addr.as_str(){                              
                                _custom_token_1 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(10000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _custom_token_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(10000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _ => {                                                             
                                    unimplemented!("{} not implemented", contract_addr.as_str().to_owned()) 
                                },
                            }
                        },
                        cosmwasm_std::WasmQuery::ContractInfo { contract_addr:_ } => unimplemented!("unimplemented"),
                        cosmwasm_std::WasmQuery::Raw { key:_, contract_addr: _ } => unimplemented!("unimplemented"),
                        _ => todo!(),
                    }
                },
                _ => todo!(),
            }
        }
    }
}
   


