use crate::state::Config;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Querier, QuerierResult, QueryRequest};
use serde::Deserialize;
use serde::Serialize;
use shadeswap_shared::amm_pair::AMMSettings;
use shadeswap_shared::{
    core::{create_viewing_key, ContractLink, TokenAmount, TokenType},
    msg::amm_pair::InitMsg,
};

pub const CONTRACT_ADDRESS: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const LP_TOKEN: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const LP_TOKEN_B: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy4";
pub const CUSTOM_TOKEN_1: &str = "secret13q9rgw3ez5mf808vm6k0naye090hh0m5fe2436";
pub const CUSTOM_TOKEN_2: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";
pub const STAKING_CONTRACT: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";
pub const FACTORY_CONTRACT_ADDRESS:& str = "secret1nulgwu6es24us9urgyvms7y02txyg0s02msgzw";
pub const SENDER:& str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
use crate::state::config_r;
use cosmwasm_std::{to_binary, Addr, DepsMut, Env, StdError, StdResult, Uint128};
use shadeswap_shared::core::ContractInstantiationInfo;

#[cfg(test)]
pub mod tests {
    use shadeswap_shared::Contract;

    use super::help_test_lib::{make_init_config, mk_amm_settings, mk_token_pair};
    use super::*;
    use crate::contract::instantiate;
    use crate::operations::{
        add_address_to_whitelist, add_whitelist_address, calculate_hash, is_address_in_whitelist,
        swap,
    };
    use crate::state::trade_count_r;
    use crate::test::help_test_lib::{
        mk_custom_token_amount, mk_native_token_pair, mock_custom_env, mock_dependencies,
    };

    #[test]
    fn assert_init_config() -> StdResult<()> {
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        let mock_info = mock_info("test", &[]);
        env.block.height = 200_000;
        env.contract.address = Addr::unchecked("ContractAddress".to_string());
        let token_pair = mk_token_pair();
        let msg = InitMsg {
            pair: token_pair,
            lp_token_contract: ContractInstantiationInfo {
                code_hash: "CODE_HASH".to_string(),
                id: 0,
            },
            factory_info: ContractLink {
                address: Addr::unchecked("FACTORYADDR"),
                code_hash: "FACTORYADDR_HASH".to_string(),
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin_auth: shadeswap_shared::Contract { address: mock_info.sender.clone(), code_hash: "".to_string() },
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };
        assert!(instantiate(deps.as_mut(), env.clone(), mock_info.clone(), msg).is_ok());
        let test_view_key =
            create_viewing_key(&env, &mock_info.clone(), seed.clone(), entropy.clone());
        // load config
        let config = config_r(deps.as_mut().storage).load()?;
        let contract_add_token_0 = match config.pair.0 {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.to_string(),
            TokenType::NativeToken { denom: _ } => "".to_string(),
        };
        assert_eq!(contract_add_token_0, CUSTOM_TOKEN_1);
        let contract_add_token_1 = match config.pair.1 {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.to_string(),
            TokenType::NativeToken { denom: _ } => "".to_string(),
        };
        assert_eq!(contract_add_token_1, CUSTOM_TOKEN_2);
        assert_eq!(test_view_key, config.viewing_key);
        Ok(())
    }

    #[test]
    fn assert_load_trade_history_first_time() -> StdResult<()> {
        let deps = mock_dependencies(&[]);
        let initial_value = match trade_count_r(&deps.storage).load() {
            Ok(it) => it,
            Err(_) => 0u64,
        };
        assert_eq!(0, initial_value);
        Ok(())
    }

    #[test]
    fn assert_add_address_to_whitelist_success() -> StdResult<()> {
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        let mock_info = mock_info("test", &[]);
        env.block.height = 200_000;
        env.contract.address = Addr::unchecked("ContractAddress".to_string());
        let token_pair = mk_token_pair();
        let msg = InitMsg {
            pair: token_pair,
            lp_token_contract: ContractInstantiationInfo {
                code_hash: "CODE_HASH".to_string(),
                id: 0,
            },
            factory_info: ContractLink {
                address: Addr::unchecked("FACTORYADDR"),
                code_hash: "FACTORYADDR_HASH".to_string(),
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin_auth: Contract { address: mock_info.sender.clone(), code_hash: "".to_string() },
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };
        assert!(instantiate(deps.as_mut(), env.clone(), mock_info.clone(), msg).is_ok());
        let address_a = Addr::unchecked("TESTA".to_string());
        let address_b = Addr::unchecked("TESTB".to_string());
        add_address_to_whitelist(deps.as_mut().storage, address_a.clone())?;
        let is_stalker_a = is_address_in_whitelist(deps.as_mut().storage, &address_a)?;
        assert_eq!(true, is_stalker_a);
        add_address_to_whitelist(deps.as_mut().storage, address_b.clone())?;
        let is_stalker_b = is_address_in_whitelist(deps.as_mut().storage, &address_b)?;
        assert_eq!(true, is_stalker_b);
        Ok(())
    }

    //#[test]
    fn assert_remove_address_from_whitelist_success() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let address_a = Addr::unchecked("TESTA".to_string());
        let address_b = Addr::unchecked("TESTB".to_string());
        let _address_c = Addr::unchecked("TESTC".to_string());
        add_whitelist_address(deps.as_mut().storage, address_a.clone())?;
        add_whitelist_address(deps.as_mut().storage, address_b.clone())?;
        Ok(())
    }

    #[test]
    fn assert_load_address_from_whitelist_success() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let address_a = Addr::unchecked("TESTA".to_string());
        let address_b = Addr::unchecked("TESTB".to_string());
        let address_c = Addr::unchecked("TESTC".to_string());
        add_whitelist_address(&mut deps.storage, address_a.clone())?;
        add_whitelist_address(&mut deps.storage, address_b.clone())?;
        add_whitelist_address(&mut deps.storage, address_c.clone())?;
        let is_addr = is_address_in_whitelist(&mut deps.storage, &address_b)?;
        assert_eq!(true, is_addr);
        let is_addr = is_address_in_whitelist(
            &mut deps.storage,
            &Addr::unchecked("TESTD".to_string()).clone(),
        )?;
        assert_eq!(false, is_addr);
        Ok(())
    }

    #[test]
    fn assert_swap_native_snip20() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let token_pair = mk_native_token_pair();
        let config = make_init_config(mk_native_token_pair().clone())?;
        let address_a = Addr::unchecked("TESTA".to_string());
        assert_eq!(
            config.factory_contract.address.as_str(),
            FACTORY_CONTRACT_ADDRESS.clone()
        );
        let router_contract = ContractLink {
            address: Addr::unchecked("router".to_string()),
            code_hash: "".to_string(),
        };
        let signature = to_binary(&"signature".to_string())?;
        let native_swap = swap(
            deps.as_mut(),
            env,
            config,
            address_a.clone(),
            None,
            mk_custom_token_amount(Uint128::from(1000u128), token_pair.clone()),
            None,
            Some(router_contract),
            Some(signature),
        )?;
        let offer_amount = &native_swap.clone().attributes[2];
        assert_eq!(offer_amount.value, 65420.to_string());
        assert_eq!(native_swap.messages.len(), 3);
        Ok(())
    }

    #[test]
    fn assert_swap_native_snip20_without_router_success() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let token_pair = mk_native_token_pair();
        let config = make_init_config(mk_native_token_pair().clone())?;
        let address_a = Addr::unchecked("TESTA".to_string());
        assert_eq!(
            config.factory_contract.address.as_str(),
            FACTORY_CONTRACT_ADDRESS.clone()
        );
        let native_swap = swap(
            deps.as_mut(),
            env,
            config,
            address_a.clone(),
            None,
            mk_custom_token_amount(Uint128::from(1000u128), token_pair.clone()),
            None,
            None,
            None,
        )?;
        let offer_amount = &native_swap.clone().attributes[2];
        assert_eq!(offer_amount.value, 65420.to_string());
        assert_eq!(native_swap.messages.len(), 2);
        Ok(())
    }

    #[test]
    fn assert_swap_native_snip20_with_router_without_signature_throws_error() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let token_pair = mk_native_token_pair();
        let config = make_init_config(mk_native_token_pair().clone())?;
        let address_a = Addr::unchecked("TESTA".to_string());
        let router_contract = ContractLink {
            address: Addr::unchecked("router".to_string()),
            code_hash: "".to_string(),
        };
        assert_eq!(
            config.factory_contract.address.as_str(),
            FACTORY_CONTRACT_ADDRESS.clone()
        );
        let native_swap = swap(
            deps.as_mut(),
            env,
            config,
            address_a.clone(),
            None,
            mk_custom_token_amount(Uint128::from(1000u128), token_pair.clone()),
            None,
            Some(router_contract),
            None,
        );
        match native_swap.unwrap_err() {
            e =>  assert_eq!(e, StdError::generic_err(
                "Callback signature needs to be passed with router contract.",
            )),
        }       
        Ok(())
    }

    #[test]
    fn assert_query_get_amm_pairs_success() -> StdResult<()> {
        let _deps = mock_dependencies(&[]);
        let _env = mock_env();
        let _amm_settings = mk_amm_settings();
        let token_pair = mk_token_pair();
        let _config = make_init_config(token_pair)?;
        let _offer_amount: u128 = 34028236692093846346337460;
        let _expected_amount: u128 = 34028236692093846346337460;
        let _address_a = "TESTA".to_string();
        Ok(())
    }

    #[test]
    pub fn assert_trader_address_hash() -> StdResult<()> {
        let trader = Addr::unchecked("test");
        let hash_address = calculate_hash(&trader.to_string());
        assert_eq!("14402189752926126668", hash_address.to_string());
        Ok(())
    }
}

#[cfg(test)]
pub mod tests_calculation_price_and_fee {
    use super::*;

    use cosmwasm_std::Decimal;

    use shadeswap_shared::core::{CustomFee, Fee, TokenPairAmount};

    use crate::operations::{
        add_liquidity, add_whitelist_address, calculate_price, calculate_swap_result, swap,
    };

    use crate::test::help_test_lib::{
        make_init_config_test_calculate_price_fee, mk_amm_settings_a,
        mk_custom_token_amount_test_calculation_price_fee,
        mk_native_token_pair_test_calculation_price_fee, mk_token_pair_test_calculation_price_fee,
        mock_custom_env, mock_dependencies,
    };

    #[test]
    fn assert_calculate_and_print_price() -> StdResult<()>{
        let result_a = Decimal::from_ratio(Uint128::from(99u128), Uint128::from(100u128)).to_string();
        let result_b = Decimal::from_ratio(Uint128::from(58u128), Uint128::from(124u128)).to_string();
        let result_c = Decimal::from_ratio(Uint128::from(158u128), Uint128::from(124u128)).to_string();
        assert_eq!(result_a, "0.99".to_string());
        assert_eq!(result_b, "0.467741935483870967".to_string());
        assert_eq!(result_c, "1.274193548387096774".to_string());
        Ok(())
    }

    #[test]
    fn assert_calculate_price() -> StdResult<()> {
        let price = calculate_price(
            Uint128::from(2000u128),
            Uint128::from(10000u128),
            Uint128::from(100000u128),
        );
        assert_eq!(Uint128::from(16666u128), price?);
        Ok(())
    }

    #[test]
    fn assert_calculate_price_sell() -> StdResult<()> {
        let price = calculate_price(
            Uint128::from(2000u128),
            Uint128::from(100000u128),
            Uint128::from(10000u128),
        );
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
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee, Some(LP_TOKEN.to_string()))?;           
        let offer_amount: u128 = 2000;
        let expected_amount: u128 = 1666;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(),&env, &amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
             Some(true));
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
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee, Some(LP_TOKEN.to_string()))?;           
        let offer_amount: u128 = 2000;
        let expected_amount: u128 = 1624;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(),&env, &amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
             None);
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
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, custom_fee, Some(LP_TOKEN.to_string()))?;           
        let offer_amount: u128 = 2000;
        let env = mock_env();
        let expected_amount: u128 = 1539;
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(), &env, &amm_settings, &config, 
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
         None);
        assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
        Ok(())
    }

    #[test]
    fn assert_calculate_swap_result_without_custom_fee() -> StdResult<()>{
        let custom_fee: Option<CustomFee> = None;
        let mut deps = mock_dependencies(&[]);
        let token_pair = mk_native_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None,Some(LP_TOKEN.to_string()))?;       
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
         None)?;
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
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None,Some(LP_TOKEN.to_string()))?;         
        let offer_amount: u128 = 2000;
        let env = mock_custom_env(FACTORY_CONTRACT_ADDRESS);
        let expected_amount: u128 = 1624;     
        let expected_lp_fee: u128 = 40;      
        let address_a = Addr::unchecked("TESTA".to_string());
        add_whitelist_address(deps.as_mut().storage, address_a.clone())?;    
        let swap_result = calculate_swap_result(deps.as_mut().as_ref(), &env,&amm_settings, &config,
            &mk_custom_token_amount_test_calculation_price_fee(Uint128::from(offer_amount), config.pair.clone()), 
            None)?;
        assert_eq!(Uint128::from(expected_amount), swap_result.result.return_amount);
        assert_eq!(Uint128::new(40u128), swap_result.lp_fee_amount);
        Ok(())
    }

    #[test]
    fn assert_slippage_swap_result_with_less_return_amount_throw_exception() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None,Some(LP_TOKEN.to_string()))?;         
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
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair, None,Some(LP_TOKEN.to_string()))?;         
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
    fn assert_slippage_add_liqudity_with_less_expected_throw_error() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None,Some(LP_TOKEN.to_string()))?;              
        let mock_info = mock_info("Sender", &[]);
        let add_liquidity_with_err = add_liquidity(
            deps.as_mut(),
            mock_env(),
            &mock_info,
            TokenPairAmount{
                pair: token_pair.clone(),
                amount_0: Uint128::from(1000000u128),
                amount_1: Uint128::from(10000u128)
            },
            Some(Uint128::from(10000001u128)),
            None
        );       

        match add_liquidity_with_err {  
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e, StdError::generic_err(
                "Operation returns less then expected (10000001 < 10000000).",
            )),
        }       
        Ok(())
    }

    #[test]
    fn assert_slippage_add_liqudity_with_equal_expected_success() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let env = mock_env();
        let mock_info = mock_info("Sender", &[]);
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None, Some(LP_TOKEN.to_string()))?;        
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
            Some(Uint128::from(10000000u128)),
            None
        )?;       
        Ok(())
    }

    #[test]
    fn assert_slippage_add_liqudity_with_more_then_expected_test_success() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let amm_settings = mk_amm_settings_a();
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let env = mock_env();
        let mock_info = mock_info("Sender", &[]);
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None, Some(LP_TOKEN.to_string()))?;        
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
            Some(Uint128::from(9999999u128)),
            None
        )?;       
        Ok(())
    }

    #[test]
    fn assert_slippage_add_liqudity_with_none_expected_slippage_success() -> StdResult<()>{
        let mut deps = mock_dependencies(&[]);
        let token_pair = mk_token_pair_test_calculation_price_fee();
        let env = mock_env();
        let mock_info = mock_info("Sender", &[]);
        let config = make_init_config_test_calculate_price_fee(deps.as_mut(), token_pair.clone(), None, Some(LP_TOKEN.to_string()))?;        
        let add_liquidity_with_success = add_liquidity(
            deps.as_mut(),
            env.clone(),
            &mock_info,
            TokenPairAmount{
                pair: token_pair.clone(),
                amount_0: Uint128::from(10000u128),
                amount_1: Uint128::from(100000u128)
            },
            None,
            None
        )?;       

        Ok(())
    }
}

pub mod help_test_lib {
    use super::*;
    use cosmwasm_std::testing::{MockApi, MockStorage};
    use cosmwasm_std::{
        from_slice, BalanceResponse, BlockInfo, Coin, ContractInfo, Empty, OwnedDeps, Timestamp,
        TransactionInfo,
    };
    use shadeswap_shared::Contract;

    use crate::contract::instantiate;
    use shadeswap_shared::core::{CustomFee, Fee, TokenPair};
    use shadeswap_shared::msg::factory::QueryResponse as FactoryQueryResponse;
    use shadeswap_shared::snip20::manager::Balance;
    use shadeswap_shared::snip20::QueryAnswer;
    use shadeswap_shared::snip20::QueryMsg;
    use shadeswap_shared::msg::staking::StakingContractInit;
    use cosmwasm_std::from_binary;
    use crate::state::config_w;

    pub fn make_init_config(token_pair: TokenPair) -> StdResult<Config> {
        let mut deps = mock_dependencies(&[]);
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let env = mock_env();
        let mock_info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let msg = InitMsg {
            pair: token_pair.clone(),
            lp_token_contract: ContractInstantiationInfo {
                code_hash: "CODE_HASH".to_string(),
                id: 0,
            },
            factory_info: ContractLink {
                address: Addr::unchecked(FACTORY_CONTRACT_ADDRESS),
                code_hash: "".to_string(),
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            admin_auth: Contract { address: mock_info.sender.clone(), code_hash: "".to_string() },
            staking_contract: None,
            custom_fee: None,
            callback: None,
        };
        assert!(instantiate(deps.as_mut(), env.clone(), mock_info.clone(), msg).is_ok());
        let config = config_r(&deps.storage).load()?;
        Ok(config)
    }

    pub fn mk_amm_settings_a() -> AMMSettings {
        AMMSettings {
            lp_fee: Fee { nom: 2, denom: 100 },
            shade_dao_fee: Fee { nom: 1, denom: 100 },
            shade_dao_address: ContractLink {
                code_hash: "CODEHAS".to_string(),
                address: Addr::unchecked("TEST".to_string()),
            },
        }
    }

    pub fn mk_token_pair() -> TokenPair {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_1.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_1.to_string(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_2.to_string(),
            },
        );
        pair
    }

    pub fn mk_native_token_pair() -> TokenPair {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string()),
                token_code_hash: CUSTOM_TOKEN_2.to_string(),
            },
            TokenType::NativeToken {
                denom: "uscrt".into(),
            },
        );
        pair
    }

    pub fn mk_custom_token_amount(amount: Uint128, token_pair: TokenPair) -> TokenAmount {
        let token = TokenAmount {
            token: token_pair.0.clone(),
            amount: amount.clone(),
        };
        token
    }

    pub fn mk_custom_token(address: String) -> TokenType {
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(address.clone()),
            token_code_hash: "TOKEN0_HASH".to_string(),
        }
    }

    pub fn mk_native_token() -> TokenType {
        TokenType::NativeToken {
            denom: "uscrt".to_string(),
        }
    }

    pub fn mk_amm_settings() -> AMMSettings {
        AMMSettings {
            shade_dao_fee: Fee { nom: 1, denom: 100 },
            lp_fee: Fee { nom: 2, denom: 100 },
            shade_dao_address: ContractLink {
                code_hash: "CODEHAS".to_string(),
                address: Addr::unchecked("TEST".to_string()),
            },
        }
    }

    pub fn mock_config(env: Env) -> StdResult<Config> {
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mk_info = mock_info("sender", &[]);

        Ok(Config {
            factory_contract: mock_contract_link(FACTORY_CONTRACT_ADDRESS.to_string()),
            lp_token: mock_contract_link("LPTOKEN".to_string()),
            staking_contract: Some(mock_contract_link(MOCK_CONTRACT_ADDR.to_string())),
            pair: mk_token_pair(),
            viewing_key: create_viewing_key(&env, &mk_info.clone(), seed.clone(), entropy.clone()),
            custom_fee: None,
            staking_contract_init: Some(StakingContractInit {
                contract_info: ContractInstantiationInfo {
                    code_hash: "".to_string(),
                    id: 1,
                },
                daily_reward_amount: Uint128::from(1000u128),
                reward_token: TokenType::CustomToken {
                    contract_addr: Addr::unchecked("".to_string()),
                    token_code_hash: "".to_string(),
                }
            }),
            prng_seed: to_binary(&"to_string".to_string())?,
            admin_auth: Contract { address: Addr::unchecked(MOCK_CONTRACT_ADDR), code_hash: "".to_string() }
        })
    }

    pub fn mock_contract_link(address: String) -> ContractLink {
        ContractLink {
            address: Addr::unchecked(address.clone()),
            code_hash: "CODEHASH".to_string(),
        }
    }

    pub fn mock_contract_info(address: &str) -> ContractLink {
        ContractLink {
            address: Addr::unchecked(address.clone()),
            code_hash: "".to_string(),
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
        _contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let _contract_addr = Addr::unchecked(MOCK_CONTRACT_ADDR);
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier { portion: 100 },
            custom_query_type: std::marker::PhantomData,
        }
    }

    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }

    pub struct MockQuerier {
        portion: u128,
    }
    impl Querier for MockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Bank(msg) => {
                    match msg {
                        cosmwasm_std::BankQuery::Balance { address, denom: _ } => {
                            match address.as_str() {
                                CUSTOM_TOKEN_2 => {
                                    let balance = to_binary(&QueryAnswer::Balance {
                                        amount: Uint128::from(1000u128),
                                    })
                                    .unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                }
                                FACTORY_CONTRACT_ADDRESS => {
                                    let balance = to_binary(&BalanceResponse {
                                        amount: Coin {
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        },
                                    })
                                    .unwrap();
                                    // let balance = to_binary(&QueryAnswer::Balance { amount: Uint128::from(1000u128)}).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                }
                                CUSTOM_TOKEN_1 => {
                                    let balance = to_binary(&BalanceResponse {
                                        amount: Coin {
                                            denom: "uscrt".into(),
                                            amount: Uint128::from(1000000u128),
                                        },
                                    })
                                    .unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                }
                                _ => {
                                    let response: &str = &address.to_string();
                                    println!("{}", response);
                                    unimplemented!("wrong tt address")
                                }
                            }
                        }
                        cosmwasm_std::BankQuery::AllBalances { address: _ } => todo!(),
                        _ => todo!(),
                    }
                }
                QueryRequest::Custom(_) => todo!(),
                QueryRequest::Wasm(msg) => match msg {
                    cosmwasm_std::WasmQuery::Smart {
                        contract_addr,
                        code_hash: _,
                        msg,
                    } => match contract_addr.as_str() {
                        FACTORY_CONTRACT_ADDRESS => {
                            let amm_settings = shadeswap_shared::amm_pair::AMMSettings {
                                lp_fee: Fee::new(28, 100),
                                shade_dao_fee: Fee::new(2, 100),
                                shade_dao_address: ContractLink {
                                    address: Addr::unchecked("DAO"),
                                    code_hash: "".to_string(),
                                },
                            };
                            let response = FactoryQueryResponse::GetConfig {
                                pair_contract: ContractInstantiationInfo { code_hash: "".to_string(), id: 1_u64 },
                                amm_settings: amm_settings,
                                lp_token_contract: ContractInstantiationInfo { code_hash: "".to_string(), id: 2_u64 },
                                authenticator: None,
                                admin_auth: Contract { address: Addr::unchecked(MOCK_CONTRACT_ADDR), code_hash: "".to_string() }
                            };
                            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&response).unwrap(),
                            ))
                        }
                        CUSTOM_TOKEN_1 => {
                            match from_binary(&msg).unwrap(){
                                QueryMsg::TokenInfo { /* fields */ } =>{
                                    let balance = to_binary(&QueryAnswer::TokenInfo { 
                                        name: "BTC".to_string(), 
                                        symbol: "BTC".to_string(), 
                                        decimals: 8, 
                                        total_supply: Some(Uint128::new(10000000)) 
                                    }).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                QueryMsg::Balance{address, key} =>{
                                    let balance = to_binary(&QueryAnswer::Balance {
                                        amount: Uint128::from(10000u128),
                                    })
                                    .unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _ =>  unimplemented!("CUSTOM_TOKEN_1")
                            }       
                           
                        }
                        CUSTOM_TOKEN_2 => {
                            match from_binary(&msg).unwrap(){
                                QueryMsg::TokenInfo { /* fields */ } =>{
                                    let balance = to_binary(&QueryAnswer::TokenInfo { 
                                        name: "ETH".to_string(), 
                                        symbol: "ETH".to_string(), 
                                        decimals: 8, 
                                        total_supply: Some(Uint128::new(10000000)) 
                                    }).unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                QueryMsg::Balance{address, key} =>{
                                    let balance = to_binary(&QueryAnswer::Balance {
                                        amount: Uint128::from(10000u128),
                                    })
                                    .unwrap();
                                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                                },
                                _ =>  unimplemented!("CUSTOM_TOKEN_2")
                            }                            
                        },
                        LP_TOKEN => {                                   
                            let balance = to_binary(&QueryAnswer::TokenInfo { 
                                name: "LPTOKEN".to_string(), 
                                symbol: "LPT".to_string(), 
                                decimals: 8, 
                                total_supply: Some(Uint128::new(10000000)) 
                            }).unwrap();
                            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))                                    
                        },
                        LP_TOKEN_B =>{
                            let balance = to_binary(&QueryAnswer::TokenInfo { 
                                name: "LPTOKEN".to_string(), 
                                symbol: "LPT".to_string(), 
                                decimals: 8, 
                                total_supply: Some(Uint128::new(0)) 
                            }).unwrap();
                            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                        },
                        _ => {        
                            println!("{}", contract_addr.as_str());                
                            unimplemented!("address not implemented")
                        },
                    },
                    cosmwasm_std::WasmQuery::ContractInfo { contract_addr: _ } => todo!(),
                    cosmwasm_std::WasmQuery::Raw {
                        key: _,
                        contract_addr: _,
                    } => todo!(),
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
    }


    pub fn make_init_config_test_calculate_price_fee(
        mut deps: DepsMut, 
        token_pair: TokenPair,
        custom_fee: Option<CustomFee>,
        lp_token_addr: Option<String>
    ) 
    -> StdResult<Config> {    
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;
        let mut deps_api = mock_dependencies(&[]);
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
            admin_auth: Contract { address: mock_info.sender.clone(), code_hash: "".to_string() },          
            staking_contract: None,
            custom_fee: custom_fee,
            callback: None,
        };         
        let temp_deps = deps.branch();
        assert!(instantiate(temp_deps, env.clone(),mock_info, msg).is_ok());
        let mut config = config_r(deps.storage).load()?;    // set staking contract        
        config.lp_token = ContractLink{
            address: deps_api.as_mut().api.addr_validate(&lp_token_addr.unwrap()).unwrap(),
            code_hash: "".to_string(),
        };
        config_w(deps.storage).save(&config).unwrap();
        Ok(config)
    }

    pub fn mk_token_pair_test_calculation_price_fee() -> TokenPair {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_1.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_1.to_string(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string().clone()),
                token_code_hash: CUSTOM_TOKEN_2.to_string(),
            },
        );
        pair
    }

    pub fn mk_custom_token_amount_test_calculation_price_fee(
        amount: Uint128,
        token_pair: TokenPair,
    ) -> TokenAmount {
        let token = TokenAmount {
            token: token_pair.0.clone(),
            amount: amount.clone(),
        };
        token
    }

    pub fn mk_native_token_pair_test_calculation_price_fee() -> TokenPair {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CUSTOM_TOKEN_2.to_string()),
                token_code_hash: CUSTOM_TOKEN_2.to_string(),
            },
            TokenType::NativeToken {
                denom: "uscrt".into(),
            },
        );
        pair
    }
}
