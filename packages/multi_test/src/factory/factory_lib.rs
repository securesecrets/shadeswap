pub mod factory_lib{
    use cosmwasm_std::{StdResult, ContractInfo, Addr, Empty, to_binary};
    use secret_multi_test::{App, Executor, Contract, ContractWrapper};
    use shadeswap_shared::Pagination;
    use shadeswap_shared::amm_pair::AMMSettings;
    use shadeswap_shared::core::{ContractInstantiationInfo, TokenPair};
    use shadeswap_shared::staking::StakingContractInit;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{amm_pair::AMMPair, factory::InitMsg};
    use shadeswap_shared::utils::asset::Contract as SContract;
    
    use crate::factory::factory_mock::factory_mock::{execute, instantiate, query, reply};
    
    use factory::contract::{execute as factory_execute, instantiate as factory_instantiate, query as factory_query};
    use shadeswap_shared::msg::factory::{ExecuteMsg, QueryMsg, QueryResponse}; 
   
    pub fn init_factory(
        router: &mut App,
        admin: &SContract,
        sender: &str,
        mock: bool,
        amm_settings: AMMSettings,
        pair: ContractInstantiationInfo,
        lp_token_info: ContractInstantiationInfo,
        seed: &str,
        api_key: &str,
        authenticator: Option<SContract>
    ) -> StdResult<ContractInfo>
    {
        if mock {
            return store_init_factory_contract(
                router,
                admin, 
                sender,
                factory_contract_store(),
                amm_settings,
                pair,
                lp_token_info,
                &seed,
                &api_key,
                authenticator
            )
        }

        return store_init_factory_contract(
            router,
            admin, 
            sender,
            factory_contract_store_in(),
            amm_settings,
            pair,
            lp_token_info,
            &seed,
            &api_key,
            authenticator
        )
    }

    pub fn store_init_factory_contract(
        router: &mut App,
        admin: &SContract,
        sender: &str,
        store_code: Box<dyn Contract<Empty>>,
        amm_settings: AMMSettings,
        pair: ContractInstantiationInfo,
        lp_token_info: ContractInstantiationInfo,
        seed: &str,
        api_key: &str,
        authenticator: Option<SContract>
    ) 
    -> StdResult<ContractInfo>
    {        
        let contract_info = router.store_code(store_code);   
        let contract = router.instantiate_contract(
            contract_info, 
            Addr::unchecked(sender.to_string()), 
            &InitMsg{
                admin_auth: admin.clone(),
                pair_contract: pair,
                amm_settings: amm_settings,
                lp_token_contract: lp_token_info,
                prng_seed: to_binary(seed)?,
                api_key: api_key.to_string(),
                authenticator: authenticator,
            }, 
            &[], 
            "staking", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)
    }

    pub fn create_amm_pairs() -> StdResult<()>{
        Ok(())
    }

    pub fn add_amm_pairs_to_factory(
        router: &mut App,
        factory_contract: &ContractInfo,
        amm_pair: &AMMPair,
        sender: &Addr
         ) -> StdResult<()>{
            let amm_pair_msg = ExecuteMsg::AddAMMPairs { 
                amm_pairs: vec![amm_pair.to_owned()] 
            };

            let _  = router.execute_contract(
                sender.to_owned(),
                
                factory_contract, 
                &amm_pair_msg,
                &[]
            ).unwrap();
        Ok(())
    }

    pub fn create_amm_pairs_to_factory(
        router: &mut App,
        factory_contract: &ContractInfo,
        token_pair: &TokenPair,
        seed: &str,
        staking_contract_info: &StakingContractInit,
        _router_contract: &ContractInfo,
        lp_token_decimals: u8,
        sender: &Addr,
         ) -> StdResult<()>{
            let create_amm_pair_msg = ExecuteMsg::CreateAMMPair { 
                pair: token_pair.to_owned(), 
                entropy: to_binary(seed)?, 
                staking_contract: Some(staking_contract_info.to_owned()),
                lp_token_decimals: lp_token_decimals,                 
            };

            let _  = router.execute_contract(
                sender.to_owned(),                
                factory_contract, 
                &create_amm_pair_msg,
                &[]
            ).unwrap();
        Ok(())
    }

    pub fn list_amm_pairs_from_factory(
        router: &mut App,
        factory_contract: &ContractInfo,
        start: u64,
        limit: u8
    ) -> StdResult<Vec<AMMPair>>{
        let query_msg = to_binary(&QueryMsg::ListAMMPairs { 
            pagination:  Pagination{
                start: start,
                limit: limit,
            }
        })?;

        let response: QueryResponse = router.query_test(factory_contract.clone(), query_msg).unwrap();
        match response {
            QueryResponse::ListAMMPairs { amm_pairs } => Ok(amm_pairs),
            _ => panic!("wrong response")
        }
    }

    pub fn factory_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }

    pub fn factory_contract_store_in() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(factory_execute, factory_instantiate, factory_query).with_reply(reply);
        Box::new(contract)
    }


}