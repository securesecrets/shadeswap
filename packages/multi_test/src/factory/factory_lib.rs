pub mod factory_lib{
    use cosmwasm_std::{StdResult, ContractInfo, Addr, Empty};
    use secret_multi_test::{App, Executor, Contract, ContractWrapper, AppResponse};
    use shadeswap_shared::amm_pair::AMMPair;
    use shadeswap_shared::utils::asset::Contract as SContract;
    use crate::factory::factory_mock::factory_mock::InitMsg;
    use crate::factory::factory_mock::factory_mock::{execute, instantiate, query};
    use factory::contract::{execute as factory_execute, instantiate as factory_instantiate, query as factory_query};
    use shadeswap_shared::msg::factory::ExecuteMsg; 

    pub fn init_factory(
        router: &mut App,
        admin: &SContract,
        sender: &str,
        mock: bool
    ) -> StdResult<ContractInfo>
    {
        if mock {
            return store_init_factory_contract(
                router,
                admin, 
                sender,
                factory_contract_store()
            )
        }

        return store_init_factory_contract(
            router,
            admin, 
            sender,
            factory_contract_store_in()
        )
    }
    pub fn store_init_factory_contract(
        router: &mut App,
        admin: &SContract,
        sender: &str,
        store_code: Box<dyn Contract<Empty>>,
    ) 
    -> StdResult<ContractInfo>
    {        
        let contract_info = router.store_code(store_code);   
        let contract = router.instantiate_contract(
            contract_info, 
            Addr::unchecked(sender.to_string()), 
            &InitMsg{
                admin_auth: admin.clone(),
            }, 
            &[], 
            "staking", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)
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

    pub fn factory_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }

    pub fn factory_contract_store_in() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(factory_execute, factory_instantiate, factory_query);
        Box::new(contract)
    }


}