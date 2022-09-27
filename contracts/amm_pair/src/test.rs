use shadeswap_shared::amm_pair::{{AMMSettings}};
use cosmwasm_std::{QuerierResult, Querier, QueryRequest};
use shadeswap_shared::{
        core::{
            create_viewing_key, ContractLink,
            TokenAmount, TokenType,
        },
        msg::amm_pair::{InitMsg}
    };
use crate::state::{{Config}};    
use serde::Deserialize;
use serde::Serialize;
use cosmwasm_std::testing::{mock_env, mock_info,MOCK_CONTRACT_ADDR};
pub const FACTORY_CONTRACT_ADDRESS: &str = "FACTORY_CONTRACT_ADDRESS";
pub const CUSTOM_TOKEN_1: &str = "CUSTOM_TOKEN_1";
pub const CUSTOM_TOKEN_2: &str = "CUSTOM_TOKEN_2";
pub const CONTRACT_ADDRESS: &str = "CONTRACT_ADDRESS";
pub const LP_TOKEN_ADDRESS: &str = "LP_TOKEN_ADDRESS";
use crate::help_math::calculate_and_print_price;
use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, StdError, StdResult, Uint128
};
use shadeswap_shared::core::{ContractInstantiationInfo};
use crate::state::config_r;

#[cfg(test)]
pub mod tests {
    use super::*;
    use super::help_test_lib::{mk_token_pair, mk_amm_settings, make_init_config};   
    use crate::contract::{instantiate};
    use crate::operations::{swap, add_whitelist_address, is_address_in_whitelist, add_address_to_whitelist, calculate_hash};
    use crate::state::{trade_count_r};
    use crate::test::help_test_lib::{mock_dependencies, mk_custom_token_amount, mk_native_token_pair, mock_custom_env};
   

    #[test]
    fn assert_init_config() -> StdResult<()> {       
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        let mock_info = mock_info("test",&[]);
        env.block.height = 200_000;
        env.contract.address = Addr::unchecked("ContractAddress".to_string());
        let token_pair = mk_token_pair();
        let msg = InitMsg {
            pair: token_pair,
            lp_token_contract: ContractInstantiationInfo{
                  code_hash: "CODE_HASH".to_string(),
                  id :0
            },
            factory_info: ContractLink {
                address: Addr::unchecked("FACTORYADDR"),
                code_hash: "FACTORYADDR_HASH".to_string()
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin: Some(mock_info.sender.clone()),           
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };     
        assert!(instantiate(deps.as_mut(), env.clone(),mock_info.clone(), msg).is_ok());     
        let test_view_key = create_viewing_key(&env, &mock_info.clone(), seed.clone(),entropy.clone());
        // load config
        let config = config_r(deps.as_mut().storage).load()?;
        let contract_add_token_0 = match(config.pair.0) {
            TokenType::CustomToken { contract_addr, token_code_hash } => contract_addr.to_string(),
            TokenType::NativeToken { denom } => "".to_string()
        };
        assert_eq!(contract_add_token_0, CUSTOM_TOKEN_1);
        let contract_add_token_1 = match(config.pair.1) {
            TokenType::CustomToken { contract_addr, token_code_hash } => contract_addr.to_string(),
            TokenType::NativeToken { denom } => "".to_string()
        };
        assert_eq!(contract_add_token_1, CUSTOM_TOKEN_2);
        assert_eq!(test_view_key, config.viewing_key);
        Ok(())
    }

 
    #[test]
    fn assert_load_trade_history_first_time() -> StdResult<()>{
        let deps = mock_dependencies(&[]);
        let initial_value = match trade_count_r(&deps.storage).load() {
            Ok(it) => it,
            Err(_) => 0u64,
        };
        assert_eq!(0, initial_value);
        Ok(())
    }
   
    #[test]
    fn assert_add_address_to_whitelist_success()-> StdResult<()>{
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        let mock_info = mock_info("test",&[]);
        env.block.height = 200_000;
        env.contract.address = Addr::unchecked("ContractAddress".to_string());
        let token_pair = mk_token_pair();
        let msg = InitMsg {
            pair: token_pair,
            lp_token_contract: ContractInstantiationInfo{
                  code_hash: "CODE_HASH".to_string(),
                  id :0
            },
            factory_info: ContractLink {
                address: Addr::unchecked("FACTORYADDR"),
                code_hash: "FACTORYADDR_HASH".to_string()
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin: Some(mock_info.sender.clone()),           
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };     
        assert!(instantiate(deps.as_mut(), env.clone(),mock_info.clone(), msg).is_ok());         
        let address_a =  Addr::unchecked("TESTA".to_string());
        let address_b =  Addr::unchecked("TESTB".to_string());
        add_address_to_whitelist(deps.as_mut().storage, address_a.clone())?;        
        let is_stalker_a = is_address_in_whitelist(deps.as_mut().storage, address_a.clone())?;
        assert_eq!(true, is_stalker_a);        
        add_address_to_whitelist(deps.as_mut().storage, address_b.clone())?;
        let is_stalker_b = is_address_in_whitelist(deps.as_mut().storage, address_b.clone())?;
        assert_eq!(true, is_stalker_b);     
        Ok(())
    }

    //#[test]
    fn assert_remove_address_from_whitelist_success()-> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let address_a =  Addr::unchecked("TESTA".to_string());
        let address_b =  Addr::unchecked("TESTB".to_string());
        let address_c =  Addr::unchecked("TESTC".to_string());
        add_whitelist_address(deps.as_mut().storage, address_a.clone())?;                    
        add_whitelist_address(deps.as_mut().storage, address_b.clone())?;
        Ok(())
    }

    
    #[test]
    fn assert_load_address_from_whitelist_success()-> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let address_a = Addr::unchecked("TESTA".to_string());
        let address_b =  Addr::unchecked("TESTB".to_string());
        let address_c =  Addr::unchecked("TESTC".to_string());
        add_whitelist_address(&mut deps.storage, address_a.clone())?;
        add_whitelist_address(&mut deps.storage, address_b.clone())?;
        add_whitelist_address(&mut deps.storage, address_c.clone())?;
        let is_addr = is_address_in_whitelist(&mut deps.storage, address_b.clone())?;  
        assert_eq!(true, is_addr);      
        let is_addr = is_address_in_whitelist(&mut deps.storage, Addr::unchecked("TESTD".to_string()).clone())?;  
        assert_eq!(false, is_addr);   
        Ok(())
    }

    #[test]
    fn assert_swap_native_snip20()-> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);        
        let token_pair = mk_native_token_pair();
        let config = make_init_config(mk_native_token_pair().clone())?;       
        let address_a = Addr::unchecked("TESTA".to_string());      
        assert_eq!(config.factory_contract.address.as_str(), FACTORY_CONTRACT_ADDRESS.clone());
        let router_contract = ContractLink{
            address: Addr::unchecked("router".to_string()),
            code_hash: "".to_string()
        };
        let signature = to_binary(&"signature".to_string())?;
        let native_swap = swap(deps.as_mut(), env, config, address_a.clone(), 
            None,  mk_custom_token_amount(Uint128::from(1000u128), token_pair.clone()),None, 
            Some(router_contract), Some(signature))?; 
        let offer_amount = &native_swap.clone().attributes[2];          
        assert_eq!(offer_amount.value, 65420.to_string());
        assert_eq!(native_swap.messages.len(), 3);
        Ok(())
    }

    #[test]
    fn assert_query_get_amm_pairs_success()-> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let amm_settings = mk_amm_settings();
        let token_pair = mk_token_pair();
        let config = make_init_config(token_pair)?;         
        let offer_amount: u128 = 34028236692093846346337460;
        let expected_amount: u128 = 34028236692093846346337460;           
        let address_a = "TESTA".to_string();
        // handle(
        //     &mut deps,
        //     env,
        //     ExecuteMsg::AddWhiteListAddress {
        //         address: Addr::unchecked(address_a.clone())
        //     },
        // )
        // .unwrap();

        // let result = query(
        //     deps.as_mut().as_ref(),
        //     env,
        //     QueryMsg::GetWhiteListAddress {
        //     },
        // )
        // .unwrap();

        // let response: QueryMsgResponse = from_binary(&result).unwrap();

        // match response {
        //     QueryMsgResponse::GetWhiteListAddress { addresses: stored } => {
        //         assert_eq!(1, stored.len())
        //     }
        //     _ => panic!("QueryResponse::ListExchanges"),
        // }
        Ok(())
    }    

    #[test]
    pub fn assert_trader_address_hash() -> StdResult<()>{
        let trader = Addr::unchecked("test");
        let hash_address = calculate_hash(&trader.to_string());
        assert_eq!("14402189752926126668", hash_address.to_string());
        Ok(())
    }
}


#[cfg(test)]
pub mod tests_calculation_price_and_fee{    
    use super::*;
    use super::help_test_lib::{mk_token_pair, mk_amm_settings, make_init_config};   
    use cosmwasm_std::{Coin, OwnedDeps, Empty, from_slice, SystemResult, SystemError, BlockInfo, Timestamp, ContractInfo, TransactionInfo, BalanceResponse, Decimal};
    use cosmwasm_std::testing::{MockStorage, MockApi, MockQuerierCustomHandlerResult, BankQuerier};
    use serde::de::DeserializeOwned;   
    use shadeswap_shared::core::{Fee, TokenPair, CustomFee, TokenPairAmount};
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
    use shadeswap_shared::router;
    use shadeswap_shared::snip20::QueryAnswer;
    use shadeswap_shared::snip20::manager::Balance;   
    use crate::contract::{instantiate, query};
    use crate::operations::{swap, add_whitelist_address, is_address_in_whitelist, add_address_to_whitelist, calculate_price, calculate_swap_result, add_liquidity};
    use crate::state::{config_w, trade_count_r};
    use crate::test::help_test_lib::{mock_dependencies, mk_custom_token_amount, mk_native_token_pair, mock_custom_env, make_init_config_test_calculate_price_fee, mk_token_pair_test_calculation_price_fee, mk_custom_token_amount_test_calculation_price_fee, mk_amm_settings_a, mk_native_token_pair_test_calculation_price_fee};

     #[test]
    fn assert_calculate_and_print_price() -> StdResult<()>{
        let result_a = calculate_and_print_price(Uint128::from(99u128), Uint128::from(100u128),0)?;
        let result_b = calculate_and_print_price(Uint128::from(58u128), Uint128::from(124u128),1)?;
        let result_c = calculate_and_print_price(Uint128::from(158u128), Uint128::from(124u128),0)?;
        assert_eq!(result_a, "0.99".to_string());
        assert_eq!(result_b, "0.467741935483870967".to_string());
        assert_eq!(result_c, "1.274193548387096774".to_string());
        Ok(())
    }

     #[test]
    fn assert_calculate_price() -> StdResult<()>{     
        let price = calculate_price(Uint128::from(2000u128), Uint128::from(10000u128), Uint128::from(100000u128));
        assert_eq!(Uint128::from(16666u128), price?);
        Ok(())
    }

    #[test]
    fn assert_calculate_price_sell() -> StdResult<()>{     
        let price = calculate_price(Uint128::from(2000u128), Uint128::from(100000u128), Uint128::from(10000u128));
        assert_eq!(Uint128::from(196u128), price?);
        Ok(())
    }
        
    #[test]
    fn assert_initial_swap_with_token_success_without_fee() -> StdResult<()>
    {     
        let custom_fee: Option<CustomFee> = None;
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee)?;           
        let offer_amount: u128 = 2000;
        let expected_amount: u128 = 1666;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(),&env, &amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
            Addr::unchecked("Test".to_string().clone()), Some(true));
        assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
        Ok(())
    }

    #[test]
    fn assert_initial_swap_with_token_success_with_fee() -> StdResult<()>
    {     
        let custom_fee: Option<CustomFee> = None;
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let env = mock_env();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee)?;           
        let offer_amount: u128 = 2000;
        let expected_amount: u128 = 1624;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(),&env, &amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
             Addr::unchecked("Test".to_string().clone()), None);
        assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
        Ok(())
    }

    #[test]
    fn assert_swap_with_custom_fee_success() -> StdResult<()>{
        let custom_fee = Some( CustomFee{
            shade_dao_fee: Fee { nom: 8, denom: 100 },
            lp_fee: Fee { nom: 1, denom: 100},
        });
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee)?;           
        let offer_amount: u128 = 2000;
        let env = mock_env();
        let expected_amount: u128 = 1539;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(), &env, &amm_settings, &config, 
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
         Addr::unchecked("Test".to_string().clone()), None);
        assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
        Ok(())
    }
        
    #[test]
    fn assert_calculate_swap_result_without_custom_fee() -> StdResult<()>{
        let custom_fee: Option<CustomFee> = None;
        let mut deps = mock_dependencies(&[]);
        let token_pair = mk_native_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None)?;       
        let address_a = Addr::unchecked("TESTA".to_string());
        let token_amount = mk_custom_token_amount_test_calculation_price_fee(Uint128::from(2000u128), config.pair.clone());   
        let amm_settings = shadeswap_shared::amm_pair::AMMSettings {
            lp_fee: Fee::new(2, 100),
            shade_dao_fee: Fee::new(3, 100),
            shade_dao_address: ContractLink {
                address: Addr::unchecked("DAO"),
                code_hash: "".to_string(),
            }
        };
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        assert_eq!(config.factory_contract.address.as_str(), FACTORY_CONTRACT_ADDRESS.clone());
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(),&env, &amm_settings, &config, &token_amount,
         address_a, None)?;
        assert_eq!(swap_result.result.return_amount, Uint128::from(159663u128));
        assert_eq!(swap_result.lp_fee_amount, Uint128::from(40u128));
        assert_eq!(swap_result.shade_dao_fee_amount, Uint128::from(60u128));
        assert_eq!(swap_result.price, "79.8315".to_string());
        Ok(())
    }

        #[test]
    fn assert_initial_swap_with_zero_fee_for_whitelist_address()-> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None)?;         
        let offer_amount: u128 = 2000;
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let expected_amount: u128 = 1666;           
        let address_a = Addr::unchecked("TESTA".to_string());
        add_whitelist_address(deps.as_mut().storage, address_a.clone())?;    
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(), &env,&amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
            address_a.clone(), None)?;
        assert_eq!(Uint128::from(expected_amount), swap_result.result.return_amount);
        assert_eq!(Uint128::zero(), swap_result.lp_fee_amount);
        Ok(())
    }

    
    #[test]
    fn assert_slippage_swap_result_with_less_return_amount_throw_exception() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None)?;         
        let offer_amount: u128 = 2000;
        let expected_amount: u128 = 16666;           
        let address_a = Addr::unchecked("TESTA".to_string());
        let token = config.pair.clone();        
        let swap_and_test_slippage = swap(
            deps.as_mut(),
            mock_custom_env(FACTORY_CONTRACT_ADDRESS),
            config,
            address_a.clone(),
            Some(address_a.clone()),          
            mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), token), 
            Some(Uint128::from(40000u128)),
            None, 
            None
        );

        match swap_and_test_slippage.unwrap_err() {
            e =>  assert_eq!(e, StdError::generic_err(
                "Operation fell short of expected_return",
            )),
        }       
        Ok(())
    }

        #[test]
    fn assert_slippage_swap_result_with_higher_return_amount_success() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None)?;         
        let offer_amount: u128 = 2000;          
        let address_a = "TESTA".to_string();
        let token = config.pair.clone();  
        let router_contract = ContractLink{
            address: Addr::unchecked("".to_string()),
            code_hash: "".to_string()
        }; 
        let signature = to_binary(&"signature".to_string())?;
        let swap_and_test_slippage = swap(
            deps.as_mut(),
            mock_custom_env(FACTORY_CONTRACT_ADDRESS),
            config,
            Addr::unchecked(address_a.clone()),
            Some(Addr::unchecked(address_a.clone())),          
            mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), token), 
            Some(Uint128::from(400u128)),
            Some(router_contract), 
            Some(signature)
        );
         assert_eq!(
            swap_and_test_slippage.unwrap().attributes[2].value, 
            1228.to_string());
        Ok(())
    }

        #[test]
    fn assert_slippage_add_liqudity_with_wrong_ration_throw_error() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None)?;         
        let offer_amount: u128 = 2000;          
        let mock_info = mock_info("Sender", &[]);
        let address_a = Addr::unchecked("TESTA".to_string());
        let token = config.pair.clone();  
        let add_liquidity_with_err = add_liquidity(
            deps.as_mut(),
            mock_env(),
            &mock_info,
            TokenPairAmount{
                pair: token_pair.clone(),
                amount_0: Uint128::from(1000000u128),
                amount_1: Uint128::from(10000u128)
            },
            Some(Decimal::percent(20)),
            None
        );       

        match add_liquidity_with_err {  
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e, StdError::generic_err(
                "Operation exceeds max slippage acceptance",
            )),
        }       
        Ok(())
    }

    #[test]
    fn assert_slippage_add_liqudity_with_right_ration_success() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let env = mock_env();
        let mock_info = mock_info("Sender", &[]);
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None)?;        
        let offer_amount: u128 = 2000;          
        let address_a = "TESTA".to_string();
        let token = config.pair.clone();  
        let add_liquidity_with_err = add_liquidity(
            deps.as_mut(),
            env.clone(),
            &mock_info,
            TokenPairAmount{
                pair: token_pair.clone(),
                amount_0: Uint128::from(10000u128),
                amount_1: Uint128::from(100000u128)
            },
            Some(Decimal::percent(20)),
            None
        );       

        match add_liquidity_with_err {  
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e, StdError::generic_err(
                "Operation exceeds max slippage acceptance",
            )),
        }       
        Ok(())
    }
}


pub mod help_test_lib {
    use super::*;   
    use cosmwasm_std::{Coin, OwnedDeps, Empty, from_slice, SystemResult, SystemError, BlockInfo, Timestamp, ContractInfo, TransactionInfo, BalanceResponse};
    use cosmwasm_std::testing::{MockStorage, MockApi, MockQuerierCustomHandlerResult, BankQuerier};
    use serde::de::DeserializeOwned;   
    use shadeswap_shared::core::{Fee, TokenPair, CustomFee};
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
    use shadeswap_shared::snip20::QueryAnswer;
    use shadeswap_shared::snip20::manager::Balance;
    use shadeswap_shared::stake_contract::StakingContractInit;   
    use crate::contract::{instantiate, query};
    use crate::operations::{swap, add_whitelist_address, is_address_in_whitelist, add_address_to_whitelist};
    use crate::state::{config_w, trade_count_r};
       
    pub fn make_init_config(   
        token_pair: TokenPair) -> StdResult<Config> {    
        let mut deps = mock_dependencies(&[]);
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let env = mock_env();  
        let mock_info = mock_info(MOCK_CONTRACT_ADDR,&[]);
        let msg = InitMsg {
            pair: token_pair.clone(),
            lp_token_contract: ContractInstantiationInfo{
                code_hash: "CODE_HASH".to_string(),
                id :0
            },
            factory_info: ContractLink {
                address: Addr::unchecked(FACTORY_CONTRACT_ADDRESS),
                code_hash: "".to_string()
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin: Some(mock_info.sender.clone()),      
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };         
        assert!(instantiate(deps.as_mut(), env.clone(), mock_info.clone(), msg).is_ok());
        let config = config_r(&deps.storage).load()?;
        Ok(config)       
    }

    pub fn mk_amm_settings_a() -> AMMSettings{
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
                address: Addr::unchecked("TEST".to_string())
            }
        }
    }

    pub fn mk_token_pair() -> TokenPair{
        let pair =  TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_1.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_1.to_string()
            },            
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_2.to_string()
            }
        );
        pair
    }

    pub fn mk_native_token_pair() -> TokenPair{
        let pair =  TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string()),
                token_code_hash: CUSTOM_TOKEN_2.to_string()
            },            
            TokenType::NativeToken {
                denom: "uscrt".into()
            }
        );
        pair
    }


    pub fn mk_custom_token_amount(amount: Uint128, token_pair: TokenPair) -> TokenAmount{    
        let token = TokenAmount{
            token: token_pair.0.clone(),
            amount: amount.clone(),
        };
        token
    }

    pub fn mk_custom_token(address: String) -> TokenType{
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(address.clone()),
            token_code_hash: "TOKEN0_HASH".to_string()
        }
    }

    pub fn mk_native_token() -> TokenType{
        TokenType::NativeToken{
            denom: "uscrt".to_string()
        }
    }

    pub fn mk_amm_settings() -> AMMSettings{
        AMMSettings{
            shade_dao_fee: Fee {
                nom: 1,
                denom: 100
            },
            lp_fee: Fee{
                nom: 2,
                denom: 100
            },
            shade_dao_address: ContractLink{
                code_hash: "CODEHAS".to_string(),
                address: Addr::unchecked("TEST".to_string())
            }
        }
    }

    pub fn mock_config(env: Env) -> StdResult<Config>
    {    
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mk_info = mock_info("sender", &[]);

        Ok(Config {       
            factory_contract: mock_contract_link(FACTORY_CONTRACT_ADDRESS.to_string()),
            lp_token: mock_contract_link("LPTOKEN".to_string()),
            staking_contract: Some(mock_contract_link(MOCK_CONTRACT_ADDR.to_string())),
            pair:      mk_token_pair(),
            viewing_key:  create_viewing_key(&env, &mk_info.clone(), seed.clone(), entropy.clone()),
            custom_fee: None,
            staking_contract_init: Some(StakingContractInit{ 
                contract_info: ContractInstantiationInfo { code_hash:"".to_string(), id: 1 }, 
                amount: Uint128::from(1000u128), 
                reward_token: TokenType::CustomToken { contract_addr: Addr::unchecked("".to_string()), token_code_hash: "".to_string() },
            }),
            prng_seed: to_binary(&"to_string".to_string())?,
        })
    }

    pub fn mock_contract_link(address: String)-> ContractLink{
        ContractLink{
            address: Addr::unchecked(address.clone()),
            code_hash: "CODEHASH".to_string()
        }
    }

    pub fn mock_contract_info(address: &str) -> ContractLink{
        ContractLink{
            address :Addr::unchecked(address.clone()),
            code_hash: "".to_string()
        }
    }

    pub fn mock_custom_env(address: &str) -> Env {
        Env {
            block: BlockInfo {
                height: 12_345,
                time: Timestamp::from_nanos(1_571_797_419_879_305_533),
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
        contract_balance: &[Coin],
      ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let contract_addr = Addr::unchecked(MOCK_CONTRACT_ADDR);
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
                        cosmwasm_std::BankQuery::Balance { address, denom } => {
                            match address.as_str() {
                                CUSTOM_TOKEN_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(1000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                FACTORY_CONTRACT_ADDRESS => {
                                    let balance = to_binary(&BalanceResponse{
                                        amount: Coin{
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        }
                                    }).unwrap();
                                    // let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(1000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                CUSTOM_TOKEN_1 => {
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
                        cosmwasm_std::BankQuery::AllBalances { address } => todo!(),
                        _ => todo!(),
                    }
                },
                QueryRequest::Custom(_) => todo!(),
                QueryRequest::Wasm(msg) =>{ 
                    match msg {
                        cosmwasm_std::WasmQuery::Smart { contract_addr, code_hash, msg } => {
                            match contract_addr.as_str(){
                                FACTORY_CONTRACT_ADDRESS => {
                                    let amm_settings = shadeswap_shared::amm_pair::AMMSettings {
                                        lp_fee: Fee::new(28, 100),
                                        shade_dao_fee: Fee::new(2, 100),
                                        shade_dao_address: ContractLink {
                                            address: Addr::unchecked("DAO"),
                                            code_hash: "".to_string(),
                                        }
                                    };
                                    let response = FactoryQueryResponse::GetAMMSettings {
                                        settings: amm_settings
                                    };
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(to_binary(&response).unwrap()))
                                },
                                CUSTOM_TOKEN_1 => {
                                    let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(10000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                CUSTOM_TOKEN_2 => {
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
                        cosmwasm_std::WasmQuery::ContractInfo { contract_addr } => todo!(),
                        cosmwasm_std::WasmQuery::Raw { key, contract_addr } => todo!(),
                        _ => todo!(),
                    }
                },
                _ => todo!(),
            }
        }
    }

    
    pub fn make_init_config_test_calculate_price_fee(
        mut deps: DepsMut, 
        token_pair: TokenPair,
        custom_fee: Option<CustomFee>,
    ) 
    -> StdResult<Config> {    
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);  
        /// let mut deps = mock_dependencies(&[]);
        let mock_info = mock_info("CONTRACT_ADDRESS",&[]);
        let msg = InitMsg {
            pair: token_pair.clone(),
            lp_token_contract: ContractInstantiationInfo{
                  code_hash: "CODE_HASH".to_string(),
                  id :0
            },
            factory_info: ContractLink {
                address: Addr::unchecked(FACTORY_CONTRACT_ADDRESS),
                code_hash: "TEST".to_string()
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin: Some(mock_info.sender.clone()),          
            staking_contract: None,
            custom_fee: custom_fee,
            callback: None,
        };         
        let temp_deps = deps.branch();
        assert!(instantiate(temp_deps, env.clone(),mock_info, msg).is_ok());
        let config = config_r(deps.storage).load()?;    // set staking contract        
        Ok(config)
    }

    pub fn mk_token_pair_test_calculation_price_fee() -> TokenPair{
        let pair =  TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_1.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_1.to_string()
            },            
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_2.to_string()
            }
        );
        pair
    }

    pub fn mk_custom_token_amount_test_calculation_price_fee(amount: Uint128, token_pair: TokenPair) -> TokenAmount{    
        let token = TokenAmount{
            token: token_pair.0.clone(),
            amount: amount.clone(),
        };
        token
    }

    pub fn mk_native_token_pair_test_calculation_price_fee() -> TokenPair{
        let pair =  TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string()),
                token_code_hash: CUSTOM_TOKEN_2.to_string()
            },            
            TokenType::NativeToken {
                denom: "uscrt".into()
            }
        );
        pair
    }    
}
